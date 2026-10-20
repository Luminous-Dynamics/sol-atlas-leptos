use sa_economic_portfolio_oracle::{
    FrontierSearchEvaluation, FrontierSearchStats, Input, Portfolio, RejectionCounts,
    check_expected, evaluate, evaluate_frontier_search,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Instant;

const DEFAULT_INPUT: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.json");
const DEFAULT_EXPECTED: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.expected.json");

#[derive(Serialize)]
struct HostMetadata {
    runner_os: Option<String>,
    runner_arch: Option<String>,
    runner_name: Option<String>,
    cpu_model: Option<String>,
    logical_cpu_count: Option<usize>,
    total_memory_bytes: Option<u64>,
    process_peak_rss_bytes: Option<u64>,
    release_build: bool,
}

#[derive(Serialize)]
struct TimingSummary {
    repetitions: usize,
    warmup_runs: usize,
    samples_elapsed_micros: Vec<u128>,
    min_elapsed_micros: u128,
    p50_elapsed_micros: u128,
    p95_elapsed_micros: u128,
    max_elapsed_micros: u128,
}

#[derive(Serialize)]
struct ComparisonBenchmarkReceipt<'a> {
    schema_version: &'static str,
    evidence_class: &'static str,
    synthetic_only: bool,
    claim_ceiling: &'static str,
    input_sha256: String,
    reference_manifest_sha256: String,
    host: HostMetadata,
    measurement_order: &'static str,
    exact_subset_count: u64,
    exact_feasible_count: usize,
    frontier_count: usize,
    exact_subset_enumeration: TimingSummary,
    dependency_aware_frontier_search: TimingSummary,
    search_stats: &'a FrontierSearchStats,
    validation: &'static str,
}

#[derive(Serialize)]
struct SearchBenchmarkReceipt<'a> {
    schema_version: String,
    evidence_class: &'static str,
    synthetic_only: bool,
    claim_ceiling: &'static str,
    input_sha256: String,
    reference_manifest_sha256: String,
    host: HostMetadata,
    timing: TimingSummary,
    stats: &'a FrontierSearchStats,
    frontier_count: usize,
    pareto_frontier: &'a [Portfolio],
    validation: &'static str,
}

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
    if args.first().map(String::as_str) == Some("--bench-search") {
        return run_search_benchmark(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("--compare-bench") {
        return run_comparison_benchmark(&args[1..]);
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

fn read_benchmark_args(args: &[String]) -> Result<(String, String, usize, String, String), String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            "usage: sa-economic-portfolio-oracle --bench-search INPUT.json REFERENCE.json [REPEATS]"
                .to_owned(),
        );
    }
    let input_path = args[0].clone();
    let reference_path = args[1].clone();
    let repetitions = if let Some(raw) = args.get(2) {
        raw.parse::<usize>()
            .map_err(|error| format!("invalid repetition count {raw}: {error}"))?
    } else {
        21
    };
    if !(1..=1_000).contains(&repetitions) {
        return Err("repetition count must be in 1..=1000".to_owned());
    }
    let input_raw = fs::read_to_string(&input_path)
        .map_err(|error| format!("cannot read input {input_path}: {error}"))?;
    let reference_raw = fs::read_to_string(&reference_path)
        .map_err(|error| format!("cannot read reference manifest {reference_path}: {error}"))?;
    Ok((
        input_path,
        reference_path,
        repetitions,
        input_raw,
        reference_raw,
    ))
}

fn first_matching_proc_line(path: &str, keys: &[&str]) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    raw.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if keys.iter().any(|candidate| key.trim() == *candidate) {
            Some(value.trim().to_owned())
        } else {
            None
        }
    })
}

fn proc_memory_bytes(path: &str, key: &str) -> Option<u64> {
    let value = first_matching_proc_line(path, &[key])?;
    let kilobytes = value.split_whitespace().next()?.parse::<u64>().ok()?;
    kilobytes.checked_mul(1024)
}

fn host_metadata() -> HostMetadata {
    HostMetadata {
        runner_os: Some(
            std::env::var("RUNNER_OS").unwrap_or_else(|_| std::env::consts::OS.to_owned()),
        ),
        runner_arch: Some(
            std::env::var("RUNNER_ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_owned()),
        ),
        runner_name: std::env::var("RUNNER_NAME").ok(),
        cpu_model: first_matching_proc_line("/proc/cpuinfo", &["model name", "Hardware"]),
        logical_cpu_count: std::thread::available_parallelism().ok().map(|count| count.get()),
        total_memory_bytes: proc_memory_bytes("/proc/meminfo", "MemTotal"),
        process_peak_rss_bytes: proc_memory_bytes("/proc/self/status", "VmHWM"),
        release_build: !cfg!(debug_assertions),
    }
}

fn percentiles(
    samples: &[u128],
    repetitions: usize,
    warmup_runs: usize,
) -> TimingSummary {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let nearest_rank = |percentile: usize| {
        let rank = (sorted.len() * percentile).div_ceil(100).max(1);
        sorted[rank - 1]
    };
    TimingSummary {
        repetitions,
        warmup_runs,
        min_elapsed_micros: *sorted.first().expect("timing sample is non-empty"),
        p50_elapsed_micros: nearest_rank(50),
        p95_elapsed_micros: nearest_rank(95),
        max_elapsed_micros: *sorted.last().expect("timing sample is non-empty"),
        samples_elapsed_micros: sorted,
    }
}

fn reference_frontier(raw: &str) -> Result<Vec<Portfolio>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|error| format!("invalid reference JSON: {error}"))?;
    let mut frontier: Vec<Portfolio> = serde_json::from_value(value["pareto_frontier"].clone())
        .map_err(|error| format!("reference frontier missing/invalid: {error}"))?;
    frontier.sort_by_key(|portfolio| portfolio.project_ids.join("+"));
    Ok(frontier)
}

fn digest_hex(raw: &str) -> String {
    Sha256::digest(raw.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_search_benchmark(args: &[String]) -> Result<(), String> {
    let (_, _, repetitions, input_raw, reference_raw) = read_benchmark_args(args)?;
    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let expected_frontier = reference_frontier(&reference_raw)?;
    let mut samples = Vec::with_capacity(repetitions);
    let mut baseline: Option<FrontierSearchEvaluation> = None;

    for _ in 0..repetitions {
        let start = Instant::now();
        let evaluation = evaluate_frontier_search(&input)?;
        samples.push(start.elapsed().as_micros());
        if evaluation.pareto_frontier != expected_frontier {
            return Err("frontier-search result does not match reference manifest".to_owned());
        }
        if let Some(previous) = &baseline {
            if previous != &evaluation {
                return Err(
                    "frontier-search result/counters changed between repeated runs".to_owned(),
                );
            }
        } else {
            baseline = Some(evaluation);
        }
    }

    let evaluation = baseline
        .as_ref()
        .expect("validated repetition count is positive");
    let timing = percentiles(&samples, repetitions);
    let schema_version = match input.schema_version.as_str() {
        "sa-economic-portfolio-v0.1" => "sa-economic-portfolio-search-benchmark-v0.1",
        "sa-economic-portfolio-v0.2" => "sa-economic-portfolio-search-benchmark-v0.2",
        _ => return Err("unsupported schema version after frontier search".to_owned()),
    };
    let receipt = SearchBenchmarkReceipt {
        schema_version: schema_version.to_owned(),
        evidence_class: "synthetic_frontier_search_benchmark",
        synthetic_only: true,
        claim_ceiling: "Repeated in-process algorithm timing on the current runner; not a cross-host SOTA claim.",
        input_sha256: digest_hex(&input_raw),
        reference_manifest_sha256: digest_hex(&reference_raw),
        timing,
        stats: &evaluation.stats,
        frontier_count: evaluation.pareto_frontier.len(),
        pareto_frontier: &evaluation.pareto_frontier,
        validation: "PASS: every repeated frontier matched the supplied reference manifest",
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_comparison_benchmark(args: &[String]) -> Result<(), String> {
    let (_, _, repetitions, input_raw, reference_raw) = read_benchmark_args(args)?;
    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let expected_frontier = reference_frontier(&reference_raw)?;
    let mut exact_samples = Vec::with_capacity(repetitions);
    let mut search_samples = Vec::with_capacity(repetitions);
    let mut exact_baseline = None;
    let mut search_baseline: Option<FrontierSearchEvaluation> = None;

    for _ in 0..repetitions {
        let exact_start = Instant::now();
        let exact = evaluate(&input)?;
        exact_samples.push(exact_start.elapsed().as_micros());
        check_expected(&exact, &reference_raw)?;
        if exact.pareto_frontier != expected_frontier {
            return Err("exact oracle frontier does not match the supplied reference".to_owned());
        }
        if let Some(previous) = &exact_baseline {
            if previous != &exact {
                return Err("exact oracle result changed between repeated runs".to_owned());
            }
        } else {
            exact_baseline = Some(exact);
        }

        let search_start = Instant::now();
        let searched = evaluate_frontier_search(&input)?;
        search_samples.push(search_start.elapsed().as_micros());
        if searched.pareto_frontier != expected_frontier {
            return Err("frontier search does not match the exact reference".to_owned());
        }
        if let Some(previous) = &search_baseline {
            if previous != &searched {
                return Err(
                    "frontier search result/counters changed between repeated runs".to_owned(),
                );
            }
        } else {
            search_baseline = Some(searched);
        }
    }

    let exact = exact_baseline
        .as_ref()
        .expect("validated repetition count is positive");
    let searched = search_baseline
        .as_ref()
        .expect("validated repetition count is positive");
    let receipt = ComparisonBenchmarkReceipt {
        schema_version: "sa-economic-portfolio-comparison-benchmark-v0.2",
        evidence_class: "synthetic_exact_vs_search_benchmark",
        synthetic_only: true,
        claim_ceiling: "Matched synthetic algorithm comparison on the current runner; not a cross-host SOTA claim.",
        input_sha256: digest_hex(&input_raw),
        reference_manifest_sha256: digest_hex(&reference_raw),
        exact_subset_count: exact.candidate_subset_count,
        exact_feasible_count: exact.feasible_portfolios.len(),
        frontier_count: exact.pareto_frontier.len(),
        exact_subset_enumeration: percentiles(&exact_samples, repetitions),
        dependency_aware_frontier_search: percentiles(&search_samples, repetitions),
        search_stats: &searched.stats,
        validation: "PASS: exact and search frontiers match the supplied manifest on every repetition",
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
            return Err(
                "frontier-search output does not match supplied reference frontier".to_owned(),
            );
        }
        let digest = Sha256::digest(reference_raw.as_bytes());
        let manifest_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        (
            Some(manifest_sha256),
            "PASS: frontier matches supplied reference manifest",
        )
    } else {
        (
            None,
            "NOT_VERIFIED: no reference frontier manifest supplied",
        )
    };

    let input_digest = Sha256::digest(input_raw.as_bytes());
    let input_sha256 = input_digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let schema_version = match input.schema_version.as_str() {
        "sa-economic-portfolio-v0.1" => "sa-economic-portfolio-search-receipt-v0.1".to_owned(),
        "sa-economic-portfolio-v0.2" => "sa-economic-portfolio-search-receipt-v0.2".to_owned(),
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
