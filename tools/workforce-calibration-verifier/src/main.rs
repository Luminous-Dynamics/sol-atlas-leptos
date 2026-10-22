use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-CALIBRATION-001";
const LOG_LOSS_FLOOR: f64 = 1e-12;
const FIXTURE_DEFAULT: &str = "docs/research/fixtures/workforce_calibration_cases_v0_1.json";
const SCHEMA_DEFAULT: &str = "docs/research/schemas/workforce_calibration_v0_1.schema.json";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Model {
    model_id: String,
    role: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Forecast {
    model_id: String,
    status: String,
    probability: Option<f64>,
    abstention_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Case {
    case_id: String,
    task_scope_id: String,
    prediction_cutoff: String,
    outcome_observed: u8,
    forecasts: Vec<Forecast>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ExpectedBin {
    bin_index: usize,
    count: usize,
    mean_predicted_probability: Option<f64>,
    observed_event_rate: Option<f64>,
    absolute_gap: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ExpectedMetrics {
    model_id: String,
    total_case_count: usize,
    scored_count: usize,
    abstained_count: usize,
    coverage: f64,
    brier_score: Option<f64>,
    log_loss: Option<f64>,
    false_positive_count: usize,
    false_negative_count: usize,
    decision_error_count: usize,
    reliability_bins: Vec<ExpectedBin>,
    passes_minimum_coverage: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    event_definition: String,
    forecast_horizon_days: u32,
    decision_threshold: f64,
    minimum_coverage: f64,
    probability_bin_width: f64,
    model_catalog: Vec<Model>,
    cases: Vec<Case>,
    expected_metrics: Vec<ExpectedMetrics>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ModelScore {
    model_id: String,
    role: String,
    total_case_count: usize,
    scored_count: usize,
    abstained_count: usize,
    coverage: f64,
    brier_score: Option<f64>,
    log_loss: Option<f64>,
    false_positive_count: usize,
    false_negative_count: usize,
    decision_error_count: usize,
    reliability_bins: Vec<ExpectedBin>,
    passes_minimum_coverage: bool,
    reference_match: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    schema_sha256: String,
    event_definition: String,
    forecast_horizon_days: u32,
    decision_threshold: f64,
    minimum_coverage: f64,
    total_case_count: usize,
    total_forecast_count: usize,
    issued_forecast_count: usize,
    abstained_forecast_count: usize,
    model_scores: Vec<ModelScore>,
    overconfidence_control_worse_on_both_proper_scores: bool,
    all_abstain_control_fails_coverage: bool,
    limitations: Vec<&'static str>,
    claim_ceiling: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rounded(value: f64) -> f64 {
    (value * 1_000_000_000.0).round() / 1_000_000_000.0
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-8
}

fn validate_schema(schema: &Value, fixture: &Value) -> Result<(), String> {
    let validator =
        validator_for(schema).map_err(|error| format!("schema cannot be compiled: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(fixture)
        .map(|error| error.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "JSON Schema validation failed:\n{}",
            errors.join("\n")
        ))
    }
}

fn validate_contract(corpus: &Corpus) -> Result<(), String> {
    let mut errors = Vec::new();
    if corpus.schema_version != EXPECTED_SCHEMA_VERSION {
        errors.push(format!("schema_version must be {EXPECTED_SCHEMA_VERSION}"));
    }
    if corpus.benchmark_id != EXPECTED_BENCHMARK_ID {
        errors.push(format!("benchmark_id must be {EXPECTED_BENCHMARK_ID}"));
    }
    if !corpus.synthetic_only || corpus.claim_ceiling != "synthetic_contract_only" {
        errors.push("v0.1 calibration corpus must remain synthetic-only".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES BELOW ARE SYNTHETIC CALIBRATION CONTRACT TEST DATA")
    {
        errors.push("fixture notice must identify synthetic calibration data".to_string());
    }
    if corpus.event_definition != "independently_confirmed_task_scoped_capacity_gap_within_horizon"
        || corpus.forecast_horizon_days == 0
        || !close(corpus.decision_threshold, 0.5)
        || !close(corpus.minimum_coverage, 0.75)
        || !close(corpus.probability_bin_width, 0.2)
    {
        errors.push("event, horizon, threshold, bin width, or coverage contract changed".to_string());
    }

    let mut model_ids = BTreeSet::new();
    for model in &corpus.model_catalog {
        if !model_ids.insert(model.model_id.as_str()) {
            errors.push(format!("duplicate model ID {}", model.model_id));
        }
    }
    let mut case_ids = BTreeSet::new();
    for case in &corpus.cases {
        if !case_ids.insert(case.case_id.as_str()) {
            errors.push(format!("duplicate case ID {}", case.case_id));
        }
        if case.outcome_observed > 1 {
            errors.push(format!("{}: outcome must be binary", case.case_id));
        }
        if case.task_scope_id.trim().is_empty() || case.prediction_cutoff.trim().is_empty() {
            errors.push(format!("{}: scope and prediction cutoff are required", case.case_id));
        }
        let mut forecast_models = BTreeSet::new();
        for forecast in &case.forecasts {
            if !forecast_models.insert(forecast.model_id.as_str()) {
                errors.push(format!(
                    "{}: duplicate forecast for model {}",
                    case.case_id, forecast.model_id
                ));
            }
            if !model_ids.contains(forecast.model_id.as_str()) {
                errors.push(format!(
                    "{}: forecast references unknown model {}",
                    case.case_id, forecast.model_id
                ));
            }
            match forecast.status.as_str() {
                "issued" => {
                    if forecast
                        .probability
                        .is_none_or(|p| !p.is_finite() || !(0.0..=1.0).contains(&p))
                        || forecast.abstention_reason.is_some()
                    {
                        errors.push(format!(
                            "{} / {}: issued forecast must have probability and no abstention reason",
                            case.case_id, forecast.model_id
                        ));
                    }
                }
                "abstained" => {
                    if forecast.probability.is_some()
                        || forecast
                            .abstention_reason
                            .as_deref()
                            .is_none_or(|reason| reason.trim().is_empty())
                    {
                        errors.push(format!(
                            "{} / {}: abstention needs null probability and explicit reason",
                            case.case_id, forecast.model_id
                        ));
                    }
                }
                _ => errors.push(format!(
                    "{} / {}: unsupported forecast status",
                    case.case_id, forecast.model_id
                )),
            }
        }
        if forecast_models != model_ids {
            errors.push(format!(
                "{}: forecast must include each catalog model exactly once",
                case.case_id
            ));
        }
    }

    let mut expected_ids = BTreeSet::new();
    for expected in &corpus.expected_metrics {
        if !expected_ids.insert(expected.model_id.as_str()) {
            errors.push(format!("duplicate expected metrics for {}", expected.model_id));
        }
    }
    if expected_ids != model_ids {
        errors.push("expected metrics must cover every model exactly once".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.model_catalog.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    corpus.cases.sort_by(|a, b| a.case_id.cmp(&b.case_id));
    for case in &mut corpus.cases {
        case.forecasts.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    }
    corpus.expected_metrics.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    for expected in &mut corpus.expected_metrics {
        expected.reliability_bins.sort_by_key(|bin| bin.bin_index);
    }
}

fn metric_matches(actual: &ModelScore, expected: &ExpectedMetrics) -> bool {
    let option_close = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => close(x, y),
        _ => false,
    };
    actual.model_id == expected.model_id
        && actual.total_case_count == expected.total_case_count
        && actual.scored_count == expected.scored_count
        && actual.abstained_count == expected.abstained_count
        && close(actual.coverage, expected.coverage)
        && option_close(actual.brier_score, expected.brier_score)
        && option_close(actual.log_loss, expected.log_loss)
        && actual.false_positive_count == expected.false_positive_count
        && actual.false_negative_count == expected.false_negative_count
        && actual.decision_error_count == expected.decision_error_count
        && actual.passes_minimum_coverage == expected.passes_minimum_coverage
        && actual.reliability_bins.len() == expected.reliability_bins.len()
        && actual
            .reliability_bins
            .iter()
            .zip(&expected.reliability_bins)
            .all(|(a, e)| {
                a.bin_index == e.bin_index
                    && a.count == e.count
                    && option_close(a.mean_predicted_probability, e.mean_predicted_probability)
                    && option_close(a.observed_event_rate, e.observed_event_rate)
                    && option_close(a.absolute_gap, e.absolute_gap)
            })
}

fn score_model(
    corpus: &Corpus,
    model: &Model,
    expected: &ExpectedMetrics,
) -> ModelScore {
    let total = corpus.cases.len();
    let mut scored: Vec<(f64, u8)> = Vec::new();
    let mut abstained_count = 0usize;
    let mut false_positive_count = 0usize;
    let mut false_negative_count = 0usize;
    let mut bins: Vec<Vec<(f64, u8)>> = vec![Vec::new(); 5];

    for case in &corpus.cases {
        let forecast = case
            .forecasts
            .iter()
            .find(|forecast| forecast.model_id == model.model_id)
            .expect("contract validation guarantees model forecast");
        if forecast.status == "abstained" {
            abstained_count += 1;
            continue;
        }
        let probability = forecast.probability.expect("issued forecast has a probability");
        scored.push((probability, case.outcome_observed));
        let bin = ((probability / corpus.probability_bin_width).floor() as usize).min(4);
        bins[bin].push((probability, case.outcome_observed));
        let predicted_event = if probability >= corpus.decision_threshold { 1 } else { 0 };
        if predicted_event == 1 && case.outcome_observed == 0 {
            false_positive_count += 1;
        }
        if predicted_event == 0 && case.outcome_observed == 1 {
            false_negative_count += 1;
        }
    }

    let scored_count = scored.len();
    let coverage = if total == 0 {
        0.0
    } else {
        scored_count as f64 / total as f64
    };
    let brier_score = if scored.is_empty() {
        None
    } else {
        Some(rounded(
            scored
                .iter()
                .map(|(p, y)| (p - f64::from(*y)).powi(2))
                .sum::<f64>()
                / scored_count as f64,
        ))
    };
    let log_loss = if scored.is_empty() {
        None
    } else {
        Some(rounded(
            scored
                .iter()
                .map(|(p, y)| {
                    let p = p.clamp(LOG_LOSS_FLOOR, 1.0 - LOG_LOSS_FLOOR);
                    -(f64::from(*y) * p.ln() + (1.0 - f64::from(*y)) * (1.0 - p).ln())
                })
                .sum::<f64>()
                / scored_count as f64,
        ))
    };

    let reliability_bins = bins
        .into_iter()
        .enumerate()
        .map(|(index, values)| {
            if values.is_empty() {
                return ExpectedBin {
                    bin_index: index,
                    count: 0,
                    mean_predicted_probability: None,
                    observed_event_rate: None,
                    absolute_gap: None,
                };
            }
            let mean_probability =
                values.iter().map(|(p, _)| p).sum::<f64>() / values.len() as f64;
            let observed_rate =
                values.iter().map(|(_, y)| usize::from(*y)).sum::<usize>() as f64
                    / values.len() as f64;
            ExpectedBin {
                bin_index: index,
                count: values.len(),
                mean_predicted_probability: Some(rounded(mean_probability)),
                observed_event_rate: Some(rounded(observed_rate)),
                absolute_gap: Some(rounded((mean_probability - observed_rate).abs())),
            }
        })
        .collect::<Vec<_>>();

    let score = ModelScore {
        model_id: model.model_id.clone(),
        role: model.role.clone(),
        total_case_count: total,
        scored_count,
        abstained_count,
        coverage: rounded(coverage),
        brier_score,
        log_loss,
        false_positive_count,
        false_negative_count,
        decision_error_count: false_positive_count + false_negative_count,
        reliability_bins,
        passes_minimum_coverage: coverage >= corpus.minimum_coverage,
        reference_match: false,
    };
    let mut score = score;
    score.reference_match = metric_matches(&score, expected);
    score
}

fn verify_values(
    schema: &Value,
    fixture: &Value,
    fixture_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<Report, String> {
    validate_schema(schema, fixture)?;
    let mut corpus: Corpus = serde_json::from_value(fixture.clone())
        .map_err(|error| format!("calibration fixture decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let mut model_scores = Vec::new();
    for model in &corpus.model_catalog {
        let expected = corpus
            .expected_metrics
            .iter()
            .find(|metrics| metrics.model_id == model.model_id)
            .expect("contract validation guarantees expected metrics");
        model_scores.push(score_model(&corpus, model, expected));
    }
    model_scores.sort_by(|a, b| a.model_id.cmp(&b.model_id));

    let all_candidates_pass_coverage = model_scores.iter().all(|score| {
        score.role != "candidate" || score.passes_minimum_coverage
    });
    let overconfident = model_scores
        .iter()
        .find(|score| score.model_id == "overconfident_control");
    let transparent = model_scores
        .iter()
        .find(|score| score.model_id == "transparent_rules");
    let overconfidence_control_worse = match (overconfident, transparent) {
        (
            Some(overconfident),
            Some(transparent),
        ) => match (
            overconfident.brier_score,
            transparent.brier_score,
            overconfident.log_loss,
            transparent.log_loss,
        ) {
            (Some(ob), Some(tb), Some(ol), Some(tl)) => ob > tb && ol > tl,
            _ => false,
        },
        _ => false,
    };
    let all_abstain_fails = model_scores
        .iter()
        .find(|score| score.model_id == "all_abstain_control")
        .is_some_and(|score| !score.passes_minimum_coverage && score.scored_count == 0);

    let total_forecast_count = corpus
        .cases
        .iter()
        .map(|case| case.forecasts.len())
        .sum::<usize>();
    let issued_forecast_count = model_scores.iter().map(|score| score.scored_count).sum();
    let abstained_forecast_count = model_scores.iter().map(|score| score.abstained_count).sum();
    let references_match = model_scores.iter().all(|score| score.reference_match);
    let status_pass = references_match
        && all_candidates_pass_coverage
        && overconfidence_control_worse
        && all_abstain_fails;

    Ok(Report {
        status: if status_pass {
            "SYNTHETIC_CALIBRATION_CONTRACT_PASS"
        } else {
            "SYNTHETIC_CALIBRATION_CONTRACT_FAIL"
        },
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: sha256(fixture_bytes),
        schema_sha256: sha256(schema_bytes),
        event_definition: corpus.event_definition,
        forecast_horizon_days: corpus.forecast_horizon_days,
        decision_threshold: corpus.decision_threshold,
        minimum_coverage: corpus.minimum_coverage,
        total_case_count: corpus.cases.len(),
        total_forecast_count,
        issued_forecast_count,
        abstained_forecast_count,
        model_scores,
        overconfidence_control_worse_on_both_proper_scores: overconfidence_control_worse,
        all_abstain_control_fails_coverage: all_abstain_fails,
        limitations: vec![
            "All cases, labels, forecasts and reference metrics are synthetic test data.",
            "Proper-score agreement validates arithmetic and contract behavior, not empirical forecasting ability.",
            "Calibration with real data requires frozen time-split held-out outcomes and uncertainty intervals.",
            "No forecast concerns a named individual or that person's employability.",
        ],
        claim_ceiling: corpus.claim_ceiling,
    })
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let fixture_path = args.next().unwrap_or_else(|| FIXTURE_DEFAULT.to_string());
    let schema_path = args.next().unwrap_or_else(|| SCHEMA_DEFAULT.to_string());
    let fixture_bytes =
        fs::read(&fixture_path).map_err(|error| format!("cannot read {fixture_path}: {error}"))?;
    let schema_bytes =
        fs::read(&schema_path).map_err(|error| format!("cannot read {schema_path}: {error}"))?;
    let fixture: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("fixture JSON is invalid: {error}"))?;
    let schema: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;
    let report = verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("report serialization failed: {error}"))?
    );
    if report.status != "SYNTHETIC_CALIBRATION_CONTRACT_PASS" {
        return Err(
            "calibration metrics or negative-control gates disagree with the frozen reference"
                .to_string(),
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const FIXTURE: &str =
        include_str!("../../../docs/research/fixtures/workforce_calibration_cases_v0_1.json");
    const SCHEMA: &str =
        include_str!("../../../docs/research/schemas/workforce_calibration_v0_1.schema.json");

    fn loaded() -> (Value, Value) {
        (
            serde_json::from_str(FIXTURE).expect("fixture parses"),
            serde_json::from_str(SCHEMA).expect("schema parses"),
        )
    }

    fn report(fixture: &Value, schema: &Value) -> Report {
        let fixture_bytes = serde_json::to_vec(fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(schema).expect("schema serializes");
        verify_values(schema, fixture, &fixture_bytes, &schema_bytes)
            .expect("synthetic calibration corpus validates")
    }

    fn find<'a>(report: &'a Report, id: &str) -> &'a ModelScore {
        report
            .model_scores
            .iter()
            .find(|score| score.model_id == id)
            .expect("model score exists")
    }

    #[test]
    fn frozen_metrics_and_negative_controls_match_reference() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(report.status, "SYNTHETIC_CALIBRATION_CONTRACT_PASS");
        assert_eq!(report.total_case_count, 12);
        assert_eq!(report.total_forecast_count, 60);
        assert_eq!(report.issued_forecast_count + report.abstained_forecast_count, 60);
    }

    #[test]
    fn proper_scores_penalize_the_overconfident_wrong_control() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let calibrated = find(&report, "transparent_rules");
        let overconfident = find(&report, "overconfident_control");
        assert!(overconfident.brier_score.unwrap() > calibrated.brier_score.unwrap());
        assert!(overconfident.log_loss.unwrap() > calibrated.log_loss.unwrap());
    }

    #[test]
    fn abstention_counts_remain_in_denominator_and_all_abstain_fails_gate() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(find(&report, "abstaining_rules").scored_count, 9);
        assert_eq!(find(&report, "abstaining_rules").abstained_count, 3);
        assert!(find(&report, "abstaining_rules").passes_minimum_coverage);
        assert_eq!(find(&report, "all_abstain_control").scored_count, 0);
        assert_eq!(find(&report, "all_abstain_control").coverage, 0.0);
        assert!(!find(&report, "all_abstain_control").passes_minimum_coverage);
    }

    #[test]
    fn row_permutation_preserves_canonical_metrics() {
        let (fixture, schema) = loaded();
        let original = report(&fixture, &schema);
        let mut permuted = fixture.clone();
        permuted["cases"].as_array_mut().expect("cases").reverse();
        for case in permuted["cases"].as_array_mut().expect("cases") {
            case["forecasts"].as_array_mut().expect("forecasts").reverse();
        }
        permuted["model_catalog"]
            .as_array_mut()
            .expect("catalog")
            .reverse();
        permuted["expected_metrics"]
            .as_array_mut()
            .expect("expected metrics")
            .reverse();
        let reordered = report(&permuted, &schema);
        assert_eq!(original.model_scores, reordered.model_scores);
    }

    #[test]
    fn duplicate_case_model_pair_is_rejected() {
        let (mut fixture, schema) = loaded();
        let duplicate = fixture["cases"][0]["forecasts"][0].clone();
        fixture["cases"][0]["forecasts"][1] = duplicate;
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn missing_outcome_is_not_silently_coerced_to_a_negative_label() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["outcome_observed"] = Value::Null;
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn malformed_abstention_is_rejected() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecasts"][0]["status"] = json!("abstained");
        fixture["cases"][0]["forecasts"][0]["probability"] = json!(0.8);
        fixture["cases"][0]["forecasts"][0]["abstention_reason"] = json!("conflict");
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn score_does_not_reward_abstaining_on_every_case() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let all_abstain = find(&report, "all_abstain_control");
        assert_eq!(all_abstain.brier_score, None);
        assert_eq!(all_abstain.log_loss, None);
        assert!(!all_abstain.passes_minimum_coverage);
    }
}
