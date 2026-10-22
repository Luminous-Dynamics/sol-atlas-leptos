use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-CALIBRATION-001";
const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_calibration_cases_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_calibration_v0_1.schema.json";
const FLOAT_TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Contract {
    event_definition: String,
    primary_metric: String,
    decision_threshold: f64,
    minimum_coverage: f64,
    log_loss_epsilon: f64,
    bin_edges: Vec<f64>,
    outcome_horizon_days: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ModelSpec {
    model_id: String,
    description: String,
    model_version: String,
    model_kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Forecast {
    model_id: String,
    decision: String,
    probability: Option<f64>,
    abstention_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Case {
    case_id: String,
    forecast_cutoff: String,
    outcome_horizon_start: String,
    outcome_horizon_end: String,
    outcome: bool,
    forecasts: Vec<Forecast>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct BinMetric {
    lower_bound: f64,
    upper_bound: f64,
    upper_inclusive: bool,
    count: usize,
    mean_probability: Option<f64>,
    observed_event_rate: Option<f64>,
    absolute_calibration_gap: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct MetricSummary {
    model_id: String,
    case_count: usize,
    issued_count: usize,
    abstained_count: usize,
    coverage: f64,
    coverage_gate_pass: bool,
    brier_score: Option<f64>,
    log_loss: Option<f64>,
    true_positive: usize,
    false_positive: usize,
    false_negative: usize,
    true_negative: usize,
    decision_error_rate: Option<f64>,
    reliability_bins: Vec<BinMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    evaluation_contract: Contract,
    models: Vec<ModelSpec>,
    cases: Vec<Case>,
    expected_metrics: Vec<MetricSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ControlCheck {
    control_id: String,
    passed: bool,
    details: String,
}

#[derive(Debug, Clone, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    schema_sha256: String,
    primary_metric: String,
    decision_threshold: f64,
    minimum_coverage: f64,
    case_count: usize,
    model_count: usize,
    metrics_match_expected: bool,
    negative_controls_pass: bool,
    metrics: Vec<MetricSummary>,
    controls: Vec<ControlCheck>,
    limitations: Vec<&'static str>,
    claim_ceiling: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= FLOAT_TOLERANCE
}

fn close_option(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(a), Some(b)) => close(a, b),
        (None, None) => true,
        _ => false,
    }
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
        errors.push(format!("schema version must be {EXPECTED_SCHEMA_VERSION}"));
    }
    if corpus.benchmark_id != EXPECTED_BENCHMARK_ID {
        errors.push(format!("benchmark ID must be {EXPECTED_BENCHMARK_ID}"));
    }
    if !corpus.synthetic_only || corpus.claim_ceiling != "synthetic_contract_only" {
        errors.push("calibration corpus must remain synthetic-only".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL CASES, FORECASTS AND OUTCOMES BELOW ARE SYNTHETIC")
    {
        errors.push("fixture notice does not establish synthetic-only scope".to_string());
    }
    if corpus.evaluation_contract.primary_metric != "brier_score" {
        errors.push("primary metric must be frozen as brier_score".to_string());
    }
    if !(0.0..=1.0).contains(&corpus.evaluation_contract.decision_threshold) {
        errors.push("decision threshold is outside [0,1]".to_string());
    }
    if !corpus.evaluation_contract.minimum_coverage.is_finite()
        || corpus.evaluation_contract.minimum_coverage <= 0.0
        || corpus.evaluation_contract.minimum_coverage > 1.0
    {
        errors.push("minimum coverage must be in (0,1]".to_string());
    }
    if !corpus.evaluation_contract.log_loss_epsilon.is_finite()
        || corpus.evaluation_contract.log_loss_epsilon <= 0.0
        || corpus.evaluation_contract.log_loss_epsilon >= 0.5
    {
        errors.push("log-loss epsilon must be in (0,0.5)".to_string());
    }
    if corpus.evaluation_contract.outcome_horizon_days == 0 {
        errors.push("outcome horizon must be positive".to_string());
    }
    if corpus.evaluation_contract.bin_edges.len() < 3
        || corpus.evaluation_contract.bin_edges.first() != Some(&0.0)
        || corpus.evaluation_contract.bin_edges.last() != Some(&1.0)
        || corpus
            .evaluation_contract
            .bin_edges
            .iter()
            .any(|edge| !edge.is_finite() || !(0.0..=1.0).contains(edge))
        || corpus
            .evaluation_contract
            .bin_edges
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        errors.push("reliability-bin edges must be strictly increasing from 0 to 1".to_string());
    }
    if corpus.models.is_empty() || corpus.cases.is_empty() {
        errors.push("models and cases must be non-empty".to_string());
    }

    let mut model_ids = BTreeSet::new();
    for model in &corpus.models {
        if !model_ids.insert(model.model_id.as_str()) {
            errors.push(format!("duplicate model ID {}", model.model_id));
        }
    }

    let mut case_ids = BTreeSet::new();
    for case in &corpus.cases {
        if !case_ids.insert(case.case_id.as_str()) {
            errors.push(format!("duplicate case ID {}", case.case_id));
        }
        if case.forecast_cutoff >= case.outcome_horizon_start
            || case.outcome_horizon_start >= case.outcome_horizon_end
        {
            errors.push(format!(
                "{}: forecast cutoff must precede a non-empty outcome horizon",
                case.case_id
            ));
        }
        if case.forecasts.len() != corpus.models.len() {
            errors.push(format!(
                "{}: expected {} forecasts, found {}",
                case.case_id,
                corpus.models.len(),
                case.forecasts.len()
            ));
        }
        let mut forecast_ids = BTreeSet::new();
        for forecast in &case.forecasts {
            if !forecast_ids.insert(forecast.model_id.as_str()) {
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
            match forecast.decision.as_str() {
                "issued" => {
                    let probability_ok = forecast
                        .probability
                        .is_some_and(|p| p.is_finite() && (0.0..=1.0).contains(&p));
                    if !probability_ok {
                        errors.push(format!(
                            "{} / {}: issued forecast requires a finite probability in [0,1]",
                            case.case_id, forecast.model_id
                        ));
                    }
                    if forecast.abstention_reason.is_some() {
                        errors.push(format!(
                            "{} / {}: issued forecast cannot carry an abstention reason",
                            case.case_id, forecast.model_id
                        ));
                    }
                }
                "abstain" => {
                    if forecast.probability.is_some() {
                        errors.push(format!(
                            "{} / {}: abstention cannot carry a probability",
                            case.case_id, forecast.model_id
                        ));
                    }
                    if !forecast
                        .abstention_reason
                        .as_deref()
                        .is_some_and(|reason| !reason.trim().is_empty())
                    {
                        errors.push(format!(
                            "{} / {}: abstention requires a reason",
                            case.case_id, forecast.model_id
                        ));
                    }
                }
                decision => errors.push(format!(
                    "{} / {}: unsupported decision {decision}",
                    case.case_id, forecast.model_id
                )),
            }
        }
        for model_id in &model_ids {
            if !forecast_ids.contains(model_id) {
                errors.push(format!(
                    "{}: missing forecast for model {model_id}",
                    case.case_id
                ));
            }
        }
    }

    let mut expected_ids = BTreeSet::new();
    for expected in &corpus.expected_metrics {
        if !expected_ids.insert(expected.model_id.as_str()) {
            errors.push(format!(
                "duplicate expected-metrics entry for {}",
                expected.model_id
            ));
        }
        if !model_ids.contains(expected.model_id.as_str()) {
            errors.push(format!(
                "expected metrics reference unknown model {}",
                expected.model_id
            ));
        }
        if expected.reliability_bins.len() + 1 != corpus.evaluation_contract.bin_edges.len() {
            errors.push(format!(
                "{}: expected reliability-bin count does not match bin edges",
                expected.model_id
            ));
        }
    }
    for model_id in &model_ids {
        if !expected_ids.contains(model_id) {
            errors.push(format!("missing expected metrics for {model_id}"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        Err(errors.join("\n"))
    }
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.models.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    corpus.cases.sort_by(|a, b| a.case_id.cmp(&b.case_id));
    for case in &mut corpus.cases {
        case.forecasts.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    }
    corpus
        .expected_metrics
        .sort_by(|a, b| a.model_id.cmp(&b.model_id));
}

fn calculate_metric(corpus: &Corpus, model: &ModelSpec) -> MetricSummary {
    let contract = &corpus.evaluation_contract;
    let mut issued: Vec<(f64, bool)> = Vec::new();
    let mut abstained_count = 0usize;

    for case in &corpus.cases {
        let forecast = case
            .forecasts
            .iter()
            .find(|forecast| forecast.model_id == model.model_id)
            .expect("validated forecast exists");
        if forecast.decision == "abstain" {
            abstained_count += 1;
        } else {
            issued.push((
                forecast.probability.expect("validated probability exists"),
                case.outcome,
            ));
        }
    }

    let case_count = corpus.cases.len();
    let issued_count = issued.len();
    let coverage = issued_count as f64 / case_count as f64;
    let brier_score = if issued.is_empty() {
        None
    } else {
        Some(
            issued
                .iter()
                .map(|(probability, outcome)| {
                    let y = if *outcome { 1.0 } else { 0.0 };
                    (probability - y).powi(2)
                })
                .sum::<f64>()
                / issued_count as f64,
        )
    };
    let epsilon = contract.log_loss_epsilon;
    let log_loss = if issued.is_empty() {
        None
    } else {
        Some(
            issued
                .iter()
                .map(|(probability, outcome)| {
                    let p = probability.clamp(epsilon, 1.0 - epsilon);
                    if *outcome { -p.ln() } else { -(1.0 - p).ln() }
                })
                .sum::<f64>()
                / issued_count as f64,
        )
    };

    let mut true_positive = 0usize;
    let mut false_positive = 0usize;
    let mut false_negative = 0usize;
    let mut true_negative = 0usize;
    for (probability, outcome) in &issued {
        let predicted_positive = *probability >= contract.decision_threshold;
        match (predicted_positive, *outcome) {
            (true, true) => true_positive += 1,
            (true, false) => false_positive += 1,
            (false, true) => false_negative += 1,
            (false, false) => true_negative += 1,
        }
    }
    let decision_error_rate = if issued_count == 0 {
        None
    } else {
        Some((false_positive + false_negative) as f64 / issued_count as f64)
    };

    let mut reliability_bins = Vec::new();
    for (index, edge_pair) in contract.bin_edges.windows(2).enumerate() {
        let lower_bound = edge_pair[0];
        let upper_bound = edge_pair[1];
        let upper_inclusive = index == contract.bin_edges.len() - 2;
        let rows: Vec<(f64, bool)> = issued
            .iter()
            .copied()
            .filter(|(probability, _)| {
                *probability >= lower_bound
                    && if upper_inclusive {
                        *probability <= upper_bound
                    } else {
                        *probability < upper_bound
                    }
            })
            .collect();
        let count = rows.len();
        let (mean_probability, observed_event_rate, absolute_calibration_gap) =
            if count == 0 {
                (None, None, None)
            } else {
                let mean = rows.iter().map(|(p, _)| *p).sum::<f64>() / count as f64;
                let event_rate =
                    rows.iter().filter(|(_, outcome)| *outcome).count() as f64 / count as f64;
                (Some(mean), Some(event_rate), Some((mean - event_rate).abs()))
            };
        reliability_bins.push(BinMetric {
            lower_bound,
            upper_bound,
            upper_inclusive,
            count,
            mean_probability,
            observed_event_rate,
            absolute_calibration_gap,
        });
    }

    MetricSummary {
        model_id: model.model_id.clone(),
        case_count,
        issued_count,
        abstained_count,
        coverage,
        coverage_gate_pass: coverage >= contract.minimum_coverage,
        brier_score,
        log_loss,
        true_positive,
        false_positive,
        false_negative,
        true_negative,
        decision_error_rate,
        reliability_bins,
    }
}

fn metric_matches(actual: &MetricSummary, expected: &MetricSummary) -> bool {
    actual.model_id == expected.model_id
        && actual.case_count == expected.case_count
        && actual.issued_count == expected.issued_count
        && actual.abstained_count == expected.abstained_count
        && close(actual.coverage, expected.coverage)
        && actual.coverage_gate_pass == expected.coverage_gate_pass
        && close_option(actual.brier_score, expected.brier_score)
        && close_option(actual.log_loss, expected.log_loss)
        && actual.true_positive == expected.true_positive
        && actual.false_positive == expected.false_positive
        && actual.false_negative == expected.false_negative
        && actual.true_negative == expected.true_negative
        && close_option(actual.decision_error_rate, expected.decision_error_rate)
        && actual.reliability_bins.len() == expected.reliability_bins.len()
        && actual
            .reliability_bins
            .iter()
            .zip(&expected.reliability_bins)
            .all(|(a, e)| {
                close(a.lower_bound, e.lower_bound)
                    && close(a.upper_bound, e.upper_bound)
                    && a.upper_inclusive == e.upper_inclusive
                    && a.count == e.count
                    && close_option(a.mean_probability, e.mean_probability)
                    && close_option(a.observed_event_rate, e.observed_event_rate)
                    && close_option(a.absolute_calibration_gap, e.absolute_calibration_gap)
            })
}

fn check_negative_controls(metrics: &[MetricSummary], minimum_coverage: f64) -> Vec<ControlCheck> {
    let find = |id: &str| metrics.iter().find(|metric| metric.model_id == id);
    let rules = find("transparent_rules");
    let overconfident = find("overconfident_wrong");
    let selective = find("selective_policy");
    let all_abstain = find("all_abstain");

    let overconfident_penalized = match (rules, overconfident) {
        (Some(r), Some(o)) => {
            o.brier_score.zip(r.brier_score).is_some_and(|(a, b)| a > b)
                && o.log_loss.zip(r.log_loss).is_some_and(|(a, b)| a > b)
        }
        _ => false,
    };
    let selective_coverage_pass = selective.is_some_and(|m| {
        m.coverage_gate_pass && m.abstained_count > 0 && m.issued_count > 0
    });
    let all_abstain_fails_gate = all_abstain.is_some_and(|m| {
        !m.coverage_gate_pass
            && close(m.coverage, 0.0)
            && m.issued_count == 0
            && m.abstained_count == m.case_count
            && m.brier_score.is_none()
            && m.log_loss.is_none()
    });
    let denominators_preserved = metrics.iter().all(|m| {
        m.case_count == m.issued_count + m.abstained_count
            && m.coverage_gate_pass == (m.coverage >= minimum_coverage)
    });

    vec![
        ControlCheck {
            control_id: "overconfident_wrong_is_penalized".to_string(),
            passed: overconfident_penalized,
            details: "Overconfident negative control must have worse Brier score and log loss than transparent rules.".to_string(),
        },
        ControlCheck {
            control_id: "selective_policy_meets_coverage_gate".to_string(),
            passed: selective_coverage_pass,
            details: "Selective policy must both abstain on some cases and issue enough forecasts to meet the frozen coverage gate.".to_string(),
        },
        ControlCheck {
            control_id: "all_abstain_fails_coverage_gate".to_string(),
            passed: all_abstain_fails_gate,
            details: "All-abstain negative control must retain all cases in the denominator and fail coverage.".to_string(),
        },
        ControlCheck {
            control_id: "denominators_and_coverage_are_complete".to_string(),
            passed: denominators_preserved,
            details: "For every model, cases equal issued plus abstained, and coverage status follows the predeclared gate.".to_string(),
        },
    ]
}

fn verify_values(
    schema: &Value,
    fixture_value: &Value,
    fixture_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<Report, String> {
    validate_schema(schema, fixture_value)?;
    let mut corpus: Corpus = serde_json::from_value(fixture_value.clone())
        .map_err(|error| format!("fixture type decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let metrics: Vec<MetricSummary> = corpus
        .models
        .iter()
        .map(|model| calculate_metric(&corpus, model))
        .collect();
    let expected_by_id = corpus
        .expected_metrics
        .iter()
        .map(|metric| (metric.model_id.as_str(), metric))
        .collect::<std::collections::BTreeMap<_, _>>();
    let metrics_match_expected = metrics.iter().all(|metric| {
        expected_by_id
            .get(metric.model_id.as_str())
            .is_some_and(|expected| metric_matches(metric, expected))
    });

    let controls = check_negative_controls(&metrics, corpus.evaluation_contract.minimum_coverage);
    let negative_controls_pass = controls.iter().all(|control| control.passed);
    let passed = metrics_match_expected && negative_controls_pass;

    Ok(Report {
        status: if passed {
            "SYNTHETIC_CALIBRATION_CONFORMANCE_PASS"
        } else {
            "SYNTHETIC_CALIBRATION_CONFORMANCE_FAIL"
        },
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: sha256(fixture_bytes),
        schema_sha256: sha256(schema_bytes),
        primary_metric: corpus.evaluation_contract.primary_metric,
        decision_threshold: corpus.evaluation_contract.decision_threshold,
        minimum_coverage: corpus.evaluation_contract.minimum_coverage,
        case_count: corpus.cases.len(),
        model_count: corpus.models.len(),
        metrics_match_expected,
        negative_controls_pass,
        metrics,
        controls,
        limitations: vec![
            "All forecasts, labels and outcomes in this corpus are synthetic contract test data.",
            "Synthetic calibration summaries do not estimate calibration on real workforce populations.",
            "The model profiles are deterministic test fixtures, not trained competing models.",
            "A passing receipt does not establish policy utility, population fairness or causal impact.",
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
    let fixture_value: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("fixture JSON is invalid: {error}"))?;
    let schema: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;

    let report = verify_values(&schema, &fixture_value, &fixture_bytes, &schema_bytes)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("report serialization failed: {error}"))?
    );
    if report.status != "SYNTHETIC_CALIBRATION_CONFORMANCE_PASS" {
        return Err("calibration metric references or negative controls did not pass".to_string());
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
            .expect("calibration fixture validates")
    }

    fn find<'a>(report: &'a Report, id: &str) -> &'a MetricSummary {
        report.metrics.iter().find(|metric| metric.model_id == id).expect("metric exists")
    }

    #[test]
    fn frozen_metrics_and_all_negative_controls_match() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(report.status, "SYNTHETIC_CALIBRATION_CONFORMANCE_PASS");
        assert_eq!(report.case_count, 12);
        assert_eq!(report.model_count, 5);
        assert!(report.metrics_match_expected);
        assert!(report.negative_controls_pass);
    }

    #[test]
    fn proper_scores_penalize_confidently_wrong_forecasts() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let rules = find(&report, "transparent_rules");
        let overconfident = find(&report, "overconfident_wrong");
        assert!(overconfident.brier_score.unwrap() > rules.brier_score.unwrap());
        assert!(overconfident.log_loss.unwrap() > rules.log_loss.unwrap());
    }

    #[test]
    fn all_abstain_is_not_a_passing_policy() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let metric = find(&report, "all_abstain");
        assert_eq!(metric.coverage, 0.0);
        assert_eq!(metric.issued_count, 0);
        assert_eq!(metric.abstained_count, 12);
        assert!(!metric.coverage_gate_pass);
        assert!(metric.brier_score.is_none());
        assert!(metric.log_loss.is_none());
    }

    #[test]
    fn selective_policy_reports_coverage_and_selective_error_separately() {
        let (fixture, schema) = loaded();
        let metric = find(&report(&fixture, &schema), "selective_policy");
        assert!(metric.coverage_gate_pass);
        assert_eq!(metric.issued_count, 8);
        assert_eq!(metric.abstained_count, 4);
        assert_eq!(metric.decision_error_rate, Some(0.0));
    }

    #[test]
    fn temporal_leakage_fails_closed() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecast_cutoff"] = json!("2026-01-02T00:00:00Z");
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn duplicate_forecasts_and_unknown_models_fail_closed() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecasts"][0]["model_id"] = json!("transparent_rules");
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());

        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecasts"][0]["model_id"] = json!("unknown_model");
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn probability_outside_unit_interval_fails_closed() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecasts"][0]["probability"] = json!(1.01);
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &bytes, &schema_bytes).is_err());
    }

    #[test]
    fn row_permutation_preserves_semantic_metrics() {
        let (fixture, schema) = loaded();
        let original = report(&fixture, &schema);
        let mut permuted = fixture.clone();
        permuted["cases"].as_array_mut().expect("cases").reverse();
        for case in permuted["cases"].as_array_mut().expect("cases") {
            case["forecasts"].as_array_mut().expect("forecasts").reverse();
        }
        permuted["models"].as_array_mut().expect("models").reverse();
        permuted["expected_metrics"]
            .as_array_mut()
            .expect("expected metrics")
            .reverse();
        let reordered = report(&permuted, &schema);
        assert_eq!(original.metrics, reordered.metrics);
        assert_eq!(original.controls, reordered.controls);
        assert_eq!(original.status, reordered.status);
    }

    #[test]
    fn exact_zero_and_one_probabilities_have_finite_log_loss() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["forecasts"][0]["probability"] = json!(1.0);
        fixture["cases"][1]["forecasts"][0]["probability"] = json!(0.0);
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        let report = verify_values(&schema, &fixture, &bytes, &schema_bytes)
            .expect("boundary probabilities remain valid");
        let metric = report.metrics.iter().find(|metric| metric.model_id == "constant_prevalence").expect("metric exists");
        assert!(metric.log_loss.unwrap().is_finite());
    }
}
