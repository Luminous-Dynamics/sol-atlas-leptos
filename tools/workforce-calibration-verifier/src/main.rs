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
const FLOAT_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct ModelSpec {
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
struct ExpectedMetric {
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
    forecast_horizon_days: u64,
    decision_threshold: f64,
    minimum_coverage: f64,
    probability_bin_width: f64,
    model_catalog: Vec<ModelSpec>,
    cases: Vec<Case>,
    expected_metrics: Vec<ExpectedMetric>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct BinMetric {
    bin_index: usize,
    count: usize,
    mean_predicted_probability: Option<f64>,
    observed_event_rate: Option<f64>,
    absolute_gap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct MetricSummary {
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
    reliability_bins: Vec<BinMetric>,
    passes_minimum_coverage: bool,
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
    primary_metric: &'static str,
    decision_threshold: f64,
    minimum_coverage: f64,
    case_count: usize,
    model_count: usize,
    reference_metrics_match: bool,
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

fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
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
    if corpus.event_definition.trim().is_empty() || corpus.forecast_horizon_days == 0 {
        errors.push("event definition and positive forecast horizon are required".to_string());
    }
    if !corpus.decision_threshold.is_finite()
        || !(0.0..=1.0).contains(&corpus.decision_threshold)
    {
        errors.push("decision threshold must be within [0,1]".to_string());
    }
    if !corpus.minimum_coverage.is_finite()
        || corpus.minimum_coverage <= 0.0
        || corpus.minimum_coverage > 1.0
    {
        errors.push("minimum coverage must be in (0,1]".to_string());
    }
    if !corpus.probability_bin_width.is_finite()
        || !close(corpus.probability_bin_width, 0.2)
    {
        errors.push("v0.1 requires the frozen reliability-bin width 0.2".to_string());
    }
    if corpus.model_catalog.is_empty() || corpus.cases.is_empty() {
        errors.push("model catalog and case set must be non-empty".to_string());
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
        if case.task_scope_id.trim().is_empty() || !valid_date(&case.prediction_cutoff) {
            errors.push(format!(
                "{}: task scope and valid prediction-cutoff date are required",
                case.case_id
            ));
        }
        if case.outcome_observed > 1 {
            errors.push(format!(
                "{}: outcome label must be binary",
                case.case_id
            ));
        }
        if case.forecasts.len() != corpus.model_catalog.len() {
            errors.push(format!(
                "{}: expected {} forecasts, found {}",
                case.case_id,
                corpus.model_catalog.len(),
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
            match forecast.status.as_str() {
                "issued" => {
                    if !forecast
                        .probability
                        .is_some_and(|p| p.is_finite() && (0.0..=1.0).contains(&p))
                    {
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
                "abstained" => {
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
                status => errors.push(format!(
                    "{} / {}: unsupported forecast status {status}",
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
        if expected.reliability_bins.len() != 5
            || expected
                .reliability_bins
                .iter()
                .enumerate()
                .any(|(index, bin)| bin.bin_index != index)
        {
            errors.push(format!(
                "{}: v0.1 requires five ordered reliability bins",
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
    corpus
        .model_catalog
        .sort_by(|a, b| a.model_id.cmp(&b.model_id));
    corpus.cases.sort_by(|a, b| a.case_id.cmp(&b.case_id));
    for case in &mut corpus.cases {
        case.forecasts.sort_by(|a, b| a.model_id.cmp(&b.model_id));
    }
    corpus
        .expected_metrics
        .sort_by(|a, b| a.model_id.cmp(&b.model_id));
}

fn calculate_metric(corpus: &Corpus, model: &ModelSpec) -> MetricSummary {
    let mut issued: Vec<(f64, u8)> = Vec::new();
    let mut abstained_count = 0usize;
    for case in &corpus.cases {
        let forecast = case
            .forecasts
            .iter()
            .find(|forecast| forecast.model_id == model.model_id)
            .expect("validated forecast exists");
        if forecast.status == "abstained" {
            abstained_count += 1;
        } else {
            issued.push((
                forecast.probability.expect("validated probability exists"),
                case.outcome_observed,
            ));
        }
    }

    let total_case_count = corpus.cases.len();
    let scored_count = issued.len();
    let coverage = scored_count as f64 / total_case_count as f64;
    let brier_score = if issued.is_empty() {
        None
    } else {
        Some(
            issued
                .iter()
                .map(|(probability, outcome)| (probability - f64::from(*outcome)).powi(2))
                .sum::<f64>()
                / scored_count as f64,
        )
    };
    let epsilon = 1e-15;
    let log_loss = if issued.is_empty() {
        None
    } else {
        Some(
            issued
                .iter()
                .map(|(probability, outcome)| {
                    let p = probability.clamp(epsilon, 1.0 - epsilon);
                    if *outcome == 1 {
                        -p.ln()
                    } else {
                        -(1.0 - p).ln()
                    }
                })
                .sum::<f64>()
                / scored_count as f64,
        )
    };

    let mut false_positive_count = 0usize;
    let mut false_negative_count = 0usize;
    for (probability, outcome) in &issued {
        let predicted_positive = *probability >= corpus.decision_threshold;
        match (predicted_positive, *outcome == 1) {
            (true, false) => false_positive_count += 1,
            (false, true) => false_negative_count += 1,
            _ => {}
        }
    }

    let mut reliability_bins = Vec::new();
    for bin_index in 0..5 {
        let lower_bound = bin_index as f64 * corpus.probability_bin_width;
        let upper_bound = if bin_index == 4 {
            1.0
        } else {
            (bin_index + 1) as f64 * corpus.probability_bin_width
        };
        let rows: Vec<(f64, u8)> = issued
            .iter()
            .copied()
            .filter(|(probability, _)| {
                *probability >= lower_bound
                    && if bin_index == 4 {
                        *probability <= upper_bound
                    } else {
                        *probability < upper_bound
                    }
            })
            .collect();
        let count = rows.len();
        let (mean_predicted_probability, observed_event_rate, absolute_gap) = if count == 0 {
            (None, None, None)
        } else {
            let mean = rows.iter().map(|(p, _)| *p).sum::<f64>() / count as f64;
            let observed = rows.iter().map(|(_, y)| usize::from(*y)).sum::<usize>() as f64
                / count as f64;
            (Some(mean), Some(observed), Some((mean - observed).abs()))
        };
        reliability_bins.push(BinMetric {
            bin_index,
            count,
            mean_predicted_probability,
            observed_event_rate,
            absolute_gap,
        });
    }

    MetricSummary {
        model_id: model.model_id.clone(),
        total_case_count,
        scored_count,
        abstained_count,
        coverage,
        brier_score,
        log_loss,
        false_positive_count,
        false_negative_count,
        decision_error_count: false_positive_count + false_negative_count,
        reliability_bins,
        passes_minimum_coverage: coverage >= corpus.minimum_coverage,
    }
}

fn metric_matches(actual: &MetricSummary, expected: &ExpectedMetric) -> bool {
    actual.model_id == expected.model_id
        && actual.total_case_count == expected.total_case_count
        && actual.scored_count == expected.scored_count
        && actual.abstained_count == expected.abstained_count
        && close(actual.coverage, expected.coverage)
        && close_option(actual.brier_score, expected.brier_score)
        && close_option(actual.log_loss, expected.log_loss)
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
                    && close_option(
                        a.mean_predicted_probability,
                        e.mean_predicted_probability,
                    )
                    && close_option(a.observed_event_rate, e.observed_event_rate)
                    && close_option(a.absolute_gap, e.absolute_gap)
            })
}

fn check_negative_controls(metrics: &[MetricSummary], minimum_coverage: f64) -> Vec<ControlCheck> {
    let find = |id: &str| metrics.iter().find(|metric| metric.model_id == id);
    let rules = find("transparent_rules");
    let overconfident = find("overconfident_control");
    let abstaining = find("abstaining_rules");
    let all_abstain = find("all_abstain_control");

    let overconfident_penalized = match (rules, overconfident) {
        (Some(rules_metric), Some(overconfident_metric)) => {
            overconfident_metric
                .brier_score
                .zip(rules_metric.brier_score)
                .is_some_and(|(overconfident_score, rules_score)| {
                    overconfident_score > rules_score
                })
                && overconfident_metric
                    .log_loss
                    .zip(rules_metric.log_loss)
                    .is_some_and(|(overconfident_score, rules_score)| {
                        overconfident_score > rules_score
                    })
        }
        _ => false,
    };
    let selective_coverage_pass = abstaining.is_some_and(|metric| {
        metric.passes_minimum_coverage && metric.abstained_count > 0 && metric.scored_count > 0
    });
    let all_abstain_fails_gate = all_abstain.is_some_and(|metric| {
        !metric.passes_minimum_coverage
            && close(metric.coverage, 0.0)
            && metric.scored_count == 0
            && metric.abstained_count == metric.total_case_count
            && metric.brier_score.is_none()
            && metric.log_loss.is_none()
    });
    let denominators_preserved = metrics.iter().all(|metric| {
        metric.total_case_count == metric.scored_count + metric.abstained_count
            && metric.passes_minimum_coverage == (metric.coverage >= minimum_coverage)
    });

    vec![
        ControlCheck {
            control_id: "overconfident_wrong_is_penalized".to_string(),
            passed: overconfident_penalized,
            details: "Overconfident control scores worse than transparent rules.".to_string(),
        },
        ControlCheck {
            control_id: "selective_policy_meets_coverage_gate".to_string(),
            passed: selective_coverage_pass,
            details: "The selective policy abstains yet meets minimum coverage.".to_string(),
        },
        ControlCheck {
            control_id: "all_abstain_fails_coverage_gate".to_string(),
            passed: all_abstain_fails_gate,
            details: "All-abstain retains the denominator but fails coverage.".to_string(),
        },
        ControlCheck {
            control_id: "denominators_and_coverage_are_complete".to_string(),
            passed: denominators_preserved,
            details: "Total equals scored plus abstained for every model.".to_string(),
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
        .model_catalog
        .iter()
        .map(|model| calculate_metric(&corpus, model))
        .collect();
    let expected_by_id = corpus
        .expected_metrics
        .iter()
        .map(|metric| (metric.model_id.as_str(), metric))
        .collect::<std::collections::BTreeMap<_, _>>();
    let reference_metrics_match = metrics.iter().all(|metric| {
        expected_by_id
            .get(metric.model_id.as_str())
            .is_some_and(|expected| metric_matches(metric, expected))
    });

    let controls = check_negative_controls(&metrics, corpus.minimum_coverage);
    let negative_controls_pass = controls.iter().all(|control| control.passed);
    let passed = reference_metrics_match && negative_controls_pass;

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
        primary_metric: "brier_score",
        decision_threshold: corpus.decision_threshold,
        minimum_coverage: corpus.minimum_coverage,
        case_count: corpus.cases.len(),
        model_count: corpus.model_catalog.len(),
        reference_metrics_match,
        negative_controls_pass,
        metrics,
        controls,
        limitations: vec![
            "All forecasts, outcomes and labels in this corpus are synthetic contract test data.",
            "Synthetic calibration does not estimate real workforce forecast skill.",
            "Model profiles are deterministic fixtures, not trained competing models.",
            "A pass does not establish real-world utility, fairness or causal impact.",
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
        return Err("calibration references or negative controls did not pass".to_string());
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
        report
            .metrics
            .iter()
            .find(|metric| metric.model_id == id)
            .expect("metric exists")
    }

    #[test]
    fn frozen_metrics_and_negative_controls_match() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(report.status, "SYNTHETIC_CALIBRATION_CONFORMANCE_PASS");
        assert_eq!(report.case_count, 12);
        assert_eq!(report.model_count, 5);
        assert!(report.reference_metrics_match);
        assert!(report.negative_controls_pass);
    }

    #[test]
    fn proper_scores_penalize_confidently_wrong_forecasts() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let rules = find(&report, "transparent_rules");
        let overconfident = find(&report, "overconfident_control");
        assert!(overconfident.brier_score.unwrap() > rules.brier_score.unwrap());
        assert!(overconfident.log_loss.unwrap() > rules.log_loss.unwrap());
    }

    #[test]
    fn all_abstain_is_not_a_passing_policy() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let metric = find(&report, "all_abstain_control");
        assert_eq!(metric.coverage, 0.0);
        assert_eq!(metric.scored_count, 0);
        assert_eq!(metric.abstained_count, 12);
        assert!(!metric.passes_minimum_coverage);
        assert!(metric.brier_score.is_none());
        assert!(metric.log_loss.is_none());
    }

    #[test]
    fn selective_policy_keeps_coverage_and_error_separate() {
        let (fixture, schema) = loaded();
        let metric = find(&report(&fixture, &schema), "abstaining_rules");
        assert!(metric.passes_minimum_coverage);
        assert_eq!(metric.scored_count, 9);
        assert_eq!(metric.abstained_count, 3);
        assert_eq!(metric.false_positive_count, 0);
        assert_eq!(metric.false_negative_count, 0);
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
    fn missing_outcome_label_is_not_converted_to_a_negative_event() {
        let (mut fixture, schema) = loaded();
        fixture["cases"][0]["outcome_observed"] = Value::Null;
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
        permuted["model_catalog"]
            .as_array_mut()
            .expect("model catalog")
            .reverse();
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
        // Deliberately make exact-certainty forecasts wrong for both labels:
        // CAL-001 is positive but gets p=0; CAL-002 is negative but gets p=1.
        fixture["cases"][0]["forecasts"][0]["probability"] = json!(0.0);
        fixture["cases"][1]["forecasts"][0]["probability"] = json!(1.0);
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        let report = verify_values(&schema, &fixture, &bytes, &schema_bytes)
            .expect("boundary probabilities remain valid");
        let metric = report
            .metrics
            .iter()
            .find(|metric| metric.model_id == "constant_prevalence")
            .expect("metric exists");
        let loss = metric.log_loss.expect("issued forecasts have log loss");
        assert!(loss.is_finite());
        assert!(loss > 30.0, "wrong exact-certainty predictions must be severely penalized");
    }
}
