//! Bounded synthetic reference oracle for post-occupancy and in-use outcomes.
//!
//! The evaluator classifies evidence and comparability. It does not calculate
//! energy savings, causal effects, regulatory compliance or resident health.
//! Meter readings, consented experience evidence and owner-native service
//! receipts remain distinct evidence classes.

use std::collections::BTreeSet;

use serde_json::{Value, json};

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn coverage_fraction(value: &Value, key: &str) -> Option<f64> {
    number(value, key).filter(|fraction| (0.0..=1.0).contains(fraction))
}

fn unique_id_set(values: Option<&[Value]>) -> Option<BTreeSet<String>> {
    let values = values?;
    if values.is_empty() {
        return None;
    }
    let mut ids = BTreeSet::new();
    for value in values {
        let id = value.as_str().filter(|id| !id.is_empty())?;
        if !ids.insert(id.to_owned()) {
            return None;
        }
    }
    Some(ids)
}

/// Compare the declared known-answer fields. Numeric JSON representations
/// compare by exact value; strings, arrays and object keys remain exact.
fn json_equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .is_some_and(|(left, right)| left == right),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| json_equivalent(left, right))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && right.iter().all(|(key, expected_value)| {
                    left.get(key)
                        .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
                })
        }
        _ => actual == expected,
    }
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    match (actual.as_object(), expected.as_object()) {
        (Some(actual), Some(expected)) => {
            actual.len() == expected.len()
                && expected.iter().all(|(key, expected_value)| {
                    actual
                        .get(key)
                        .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
                })
        }
        _ => false,
    }
}

fn unsupported_result(case_id: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "disposition": "Unsupported",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "metric_results": [],
        "measurement_and_verification_status": "NotClaimed",
        "causal_assessment_status": "NotClaimed",
        "claim_ceiling": "synthetic outcome classification only; not measured building performance, regulatory compliance, energy savings, occupant wellbeing, or causal impact"
    })
}

fn same_scope_and_context(observation: &Value, scope: &Value) -> bool {
    observation.get("scope_digest") == scope.get("scope_digest")
        && observation.get("design_revision") == scope.get("design_revision")
        && observation.get("bom_revision") == scope.get("bom_revision")
        && observation.get("observation_window") == scope.get("observation_window")
        && observation.get("context") == scope.get("analysis_context")
}

fn owner_binding_matches(case_id: &str, requirement: &Value, observation: &Value) -> bool {
    text(observation, "source_owner") == text(requirement, "expected_source_owner")
        && text(observation, "source_contract") == text(requirement, "expected_source_contract")
        && text(observation, "source_revision") == text(requirement, "expected_source_revision")
        && !text(observation, "source_receipt_id").is_empty()
        && text(observation, "source_subject_digest")
            == format!(
                "synthetic-subject:{case_id}:{}",
                text(requirement, "metric_id")
            )
}

fn evaluate_static_prediction(
    observation: &Value,
    prediction: Option<&Value>,
    requirement: &Value,
    scope: &Value,
) -> (String, Option<f64>, Option<String>) {
    let Some(prediction) = prediction else {
        return ("Unknown".to_owned(), None, None);
    };

    if text(prediction, "unit") != text(requirement, "unit") {
        return (
            "Unsupported".to_owned(),
            None,
            Some("STATIC_PREDICTION_UNIT_MISMATCH".to_owned()),
        );
    }
    if text(prediction, "evidence_state") != "current" {
        return (
            "Unknown".to_owned(),
            None,
            Some("STATIC_PREDICTION_NOT_CURRENT".to_owned()),
        );
    }
    if prediction.get("scope_digest") != scope.get("scope_digest")
        || prediction.get("design_revision") != scope.get("design_revision")
        || prediction.get("bom_revision") != scope.get("bom_revision")
        || prediction.get("context") != scope.get("analysis_context")
    {
        return (
            "NotComparable".to_owned(),
            None,
            Some("STATIC_PREDICTION_CONTEXT_OR_REVISION_MISMATCH".to_owned()),
        );
    }
    let Some(observed_value) = number(observation, "value") else {
        return (
            "Unsupported".to_owned(),
            None,
            Some("OBSERVATION_VALUE_INVALID".to_owned()),
        );
    };
    let Some(predicted_value) = number(prediction, "value") else {
        return (
            "Unsupported".to_owned(),
            None,
            Some("STATIC_PREDICTION_VALUE_INVALID".to_owned()),
        );
    };

    (
        "Comparable".to_owned(),
        Some(observed_value - predicted_value),
        None,
    )
}

fn evaluate_dynamic_model(
    observation: &Value,
    model: Option<&Value>,
    requirement: &Value,
    scope: &Value,
) -> (String, Option<f64>, Option<String>) {
    let Some(model) = model else {
        return ("Unknown".to_owned(), None, None);
    };

    if text(model, "unit") != text(requirement, "unit") {
        return (
            "Unsupported".to_owned(),
            None,
            Some("DYNAMIC_MODEL_UNIT_MISMATCH".to_owned()),
        );
    }

    let training_digest = text(model, "training_data_digest");
    let validation_digest = text(model, "validation_data_digest");
    let training_ids = unique_id_set(array(model, "training_observation_ids"));
    let validation_ids = unique_id_set(array(model, "validation_observation_ids"));
    let split_is_disjoint = training_ids
        .as_ref()
        .zip(validation_ids.as_ref())
        .is_some_and(|(training, validation)| validation.iter().all(|id| !training.contains(id)));
    if text(model, "evidence_state") != "current"
        || text(model, "calibration_status") != "calibrated"
        || text(model, "independent_validation") != "passed"
        || training_digest.is_empty()
        || validation_digest.is_empty()
        || training_digest == validation_digest
        || !split_is_disjoint
        || text(model, "validation_protocol_id").is_empty()
        || text(model, "partition_audit_receipt_id").is_empty()
        || model
            .get("envelope_contains_observation")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return (
            "Unknown".to_owned(),
            None,
            Some("DYNAMIC_MODEL_VALIDATION_OR_APPLICABILITY_INSUFFICIENT".to_owned()),
        );
    }

    if model.get("scope_digest") != scope.get("scope_digest")
        || model.get("design_revision") != scope.get("design_revision")
        || model.get("bom_revision") != scope.get("bom_revision")
        || model.get("context") != scope.get("analysis_context")
    {
        return (
            "NotComparable".to_owned(),
            None,
            Some("DYNAMIC_MODEL_CONTEXT_OR_REVISION_MISMATCH".to_owned()),
        );
    }

    let Some(observed_value) = number(observation, "value") else {
        return (
            "Unsupported".to_owned(),
            None,
            Some("OBSERVATION_VALUE_INVALID".to_owned()),
        );
    };
    let Some(model_value) = number(model, "value") else {
        return (
            "Unsupported".to_owned(),
            None,
            Some("DYNAMIC_MODEL_VALUE_INVALID".to_owned()),
        );
    };

    (
        "Comparable".to_owned(),
        Some(observed_value - model_value),
        None,
    )
}

fn baseline_and_post_windows_are_valid(mv: &Value, scope: &Value) -> bool {
    let Some(baseline) = mv.get("baseline_window") else {
        return false;
    };
    let Some(post) = mv.get("post_window") else {
        return false;
    };
    let Some(scope_window) = scope.get("observation_window") else {
        return false;
    };

    let baseline_start = baseline.get("start_tick").and_then(Value::as_i64);
    let baseline_end = baseline.get("end_tick").and_then(Value::as_i64);
    let post_start = post.get("start_tick").and_then(Value::as_i64);
    let post_end = post.get("end_tick").and_then(Value::as_i64);
    let scope_start = scope_window.get("start_tick").and_then(Value::as_i64);
    let scope_end = scope_window.get("end_tick").and_then(Value::as_i64);
    let unit = text(scope_window, "tick_unit");

    baseline_start
        .zip(baseline_end)
        .zip(post_start.zip(post_end))
        .zip(scope_start.zip(scope_end))
        .is_some_and(
            |(
                ((baseline_start, baseline_end), (post_start, post_end)),
                (scope_start, scope_end),
            )| {
                baseline_end > baseline_start
                    && post_end > post_start
                    && baseline_end <= post_start
                    && post_start == scope_start
                    && post_end == scope_end
                    && text(baseline, "window_id") == text(mv, "baseline_period")
                    && text(post, "window_id") == text(mv, "post_period")
                    && text(post, "window_id") == text(scope_window, "window_id")
                    && !unit.is_empty()
                    && text(baseline, "tick_unit") == unit
                    && text(post, "tick_unit") == unit
            },
        )
}

fn classify_measurement_and_verification(
    case_id: &str,
    input: &Value,
    scope: &Value,
) -> &'static str {
    let Some(mv) = input.get("measurement_and_verification") else {
        return "NotClaimed";
    };
    if mv.get("claim_requested").and_then(Value::as_bool) != Some(true) {
        return "NotClaimed";
    }

    let required_normalization = array(scope, "normalization_variables").unwrap_or(&[]);
    let supplied_normalization = array(mv, "normalized_for").unwrap_or(&[]);
    let normalized = required_normalization.iter().all(|required| {
        supplied_normalization
            .iter()
            .any(|supplied| supplied == required)
    });
    let coverage = coverage_fraction(mv, "coverage_fraction").unwrap_or(-1.0);
    let minimum_coverage = number(scope, "minimum_coverage_fraction").unwrap_or(1.0);
    let baseline_period = text(mv, "baseline_period");
    let post_period = text(mv, "post_period");
    let post_receipt_id = text(mv, "post_receipt_id");
    let post_metric_id = text(mv, "post_metric_id");
    let post_requirement = array(input, "requirements")
        .unwrap_or(&[])
        .iter()
        .find(|requirement| text(requirement, "metric_id") == post_metric_id);
    let post_observation = array(input, "observations")
        .unwrap_or(&[])
        .iter()
        .find(|observation| {
            text(observation, "metric_id") == post_metric_id
                && text(observation, "source_receipt_id") == post_receipt_id
        });
    let post_receipt_matches =
        post_requirement
            .zip(post_observation)
            .is_some_and(|(requirement, observation)| {
                text(requirement, "evidence_class") == "meter"
                    && text(requirement, "unit") == text(observation, "unit")
                    && text(observation, "evidence_state") == "current"
                    && text(observation, "evidence_kind") == "meter_measurement"
                    && text(observation, "calibration_state") == "current"
                    && !text(observation, "instrument_id").is_empty()
                    && coverage_fraction(observation, "coverage_fraction")
                        .is_some_and(|value| value >= minimum_coverage)
                    && same_scope_and_context(observation, scope)
                    && number(observation, "value").is_some()
                    && owner_binding_matches(case_id, requirement, observation)
            });

    if text(mv, "evidence_state") == "current"
        && !text(mv, "method_id").is_empty()
        && !text(mv, "method_revision").is_empty()
        && !text(mv, "baseline_receipt_id").is_empty()
        && post_receipt_matches
        && !baseline_period.is_empty()
        && !post_period.is_empty()
        && baseline_period != post_period
        && baseline_and_post_windows_are_valid(mv, scope)
        && normalized
        && coverage >= minimum_coverage
        && text(mv, "baseline_adjustment_review") == "passed"
    {
        "EligibleForSavingsReview"
    } else {
        "Unsupported"
    }
}

fn classify_causal_assessment(input: &Value) -> &'static str {
    let Some(causal) = input.get("causal_assessment") else {
        return "NotClaimed";
    };
    if causal.get("claim_requested").and_then(Value::as_bool) != Some(true) {
        return "NotClaimed";
    }

    if text(causal, "design") == "matched_control_pre_registered"
        && !text(causal, "control_group_id").is_empty()
        && !text(causal, "preregistration_digest").is_empty()
        && text(causal, "assumption_review") == "passed"
        && text(causal, "independent_review") == "passed"
    {
        "EligibleForIndependentCausalReview"
    } else {
        "Unsupported"
    }
}

/// Evaluate one synthetic in-use outcome case from input only. The expected
/// known-answer field is never read by this function.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let scope = input.get("scope").unwrap_or(&Value::Null);
    let mut unsupported = BTreeSet::<String>::new();

    if text(scope, "scope_digest").is_empty()
        || text(scope, "design_revision").is_empty()
        || text(scope, "bom_revision").is_empty()
    {
        unsupported.insert("SCOPE_OR_DESIGN_IDENTITY_MISSING".to_owned());
    }
    let Some(window) = scope.get("observation_window") else {
        unsupported.insert("SCOPE_OBSERVATION_WINDOW_MISSING".to_owned());
        return unsupported_result(case_id, unsupported);
    };
    let start_tick = window.get("start_tick").and_then(Value::as_i64);
    let end_tick = window.get("end_tick").and_then(Value::as_i64);
    if start_tick
        .zip(end_tick)
        .is_none_or(|(start, end)| end <= start)
        || text(window, "window_id").is_empty()
        || text(window, "tick_unit").is_empty()
    {
        unsupported.insert("SCOPE_OBSERVATION_WINDOW_INVALID".to_owned());
    }
    if scope
        .get("analysis_context")
        .and_then(Value::as_object)
        .is_none()
    {
        unsupported.insert("SCOPE_ANALYSIS_CONTEXT_MISSING".to_owned());
    }
    let minimum_coverage = number(scope, "minimum_coverage_fraction");
    if minimum_coverage.is_none_or(|value| !(0.0..=1.0).contains(&value)) {
        unsupported.insert("SCOPE_MINIMUM_COVERAGE_INVALID".to_owned());
    }

    let Some(requirements) = array(input, "requirements") else {
        unsupported.insert("REQUIREMENTS_MISSING".to_owned());
        return unsupported_result(case_id, unsupported);
    };
    if requirements.is_empty() {
        unsupported.insert("REQUIREMENTS_EMPTY".to_owned());
    }
    let Some(observations) = array(input, "observations") else {
        unsupported.insert("OBSERVATIONS_MISSING".to_owned());
        return unsupported_result(case_id, unsupported);
    };

    let mut requirement_ids = BTreeSet::<String>::new();
    for requirement in requirements {
        let metric_id = text(requirement, "metric_id");
        if metric_id.is_empty() || !requirement_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_OR_MISSING_REQUIREMENT_METRIC_ID".to_owned());
        }
        if text(requirement, "unit").is_empty()
            || !matches!(text(requirement, "direction"), "le" | "ge")
            || number(requirement, "threshold").is_none()
            || text(requirement, "definition_revision").is_empty()
            || !matches!(
                text(requirement, "evidence_class"),
                "meter" | "experience" | "service"
            )
            || text(requirement, "expected_source_owner").is_empty()
            || text(requirement, "expected_source_contract").is_empty()
            || text(requirement, "expected_source_revision").is_empty()
        {
            unsupported.insert("REQUIREMENT_DEFINITION_INVALID".to_owned());
        }
    }

    let mut observation_ids = BTreeSet::<String>::new();
    for observation in observations {
        let metric_id = text(observation, "metric_id");
        if metric_id.is_empty() || !observation_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_OBSERVATION_METRIC_ID".to_owned());
        }
        if !requirement_ids.contains(metric_id) {
            unsupported.insert("OBSERVATION_METRIC_UNDEFINED".to_owned());
        }
    }

    let mut prediction_ids = BTreeSet::<String>::new();
    for prediction in array(input, "design_predictions").unwrap_or(&[]) {
        let metric_id = text(prediction, "metric_id");
        if metric_id.is_empty() || !prediction_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_OR_MISSING_STATIC_PREDICTION_METRIC_ID".to_owned());
        }
        if !requirement_ids.contains(metric_id) {
            unsupported.insert("STATIC_PREDICTION_METRIC_UNDEFINED".to_owned());
        }
    }

    let mut dynamic_ids = BTreeSet::<String>::new();
    for model in array(input, "calibrated_models").unwrap_or(&[]) {
        let metric_id = text(model, "metric_id");
        if metric_id.is_empty() || !dynamic_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_OR_MISSING_DYNAMIC_MODEL_METRIC_ID".to_owned());
        }
        if !requirement_ids.contains(metric_id) {
            unsupported.insert("DYNAMIC_MODEL_METRIC_UNDEFINED".to_owned());
        }
    }

    if !unsupported.is_empty() {
        return unsupported_result(case_id, unsupported);
    }

    let mut metric_results = Vec::<Value>::new();
    for requirement in requirements {
        let metric_id = text(requirement, "metric_id");
        let mut reasons = BTreeSet::<String>::new();
        let Some(observation) = observations
            .iter()
            .find(|observation| text(observation, "metric_id") == metric_id)
        else {
            metric_results.push(json!({
                "metric_id": metric_id,
                "observation_state": "Unknown",
                "threshold_state": "Unknown",
                "observed_value": null,
                "unit": text(requirement, "unit"),
                "static_gap_state": "Unknown",
                "static_gap_value": null,
                "dynamic_gap_state": "Unknown",
                "dynamic_gap_value": null,
                "reason_codes": ["OBSERVATION_MISSING"]
            }));
            continue;
        };

        let mut observation_state = "Current";
        let mut threshold_state = "Unknown";
        let mut observed_value = None;
        let mut static_gap_state = "Unknown".to_owned();
        let mut static_gap_value = None;
        let mut dynamic_gap_state = "Unknown".to_owned();
        let mut dynamic_gap_value = None;

        if text(observation, "unit") != text(requirement, "unit") {
            observation_state = "Unsupported";
            reasons.insert("OBSERVATION_UNIT_MISMATCH".to_owned());
        } else if !same_scope_and_context(observation, scope) {
            observation_state = "NotComparable";
            reasons.insert("OBSERVATION_SCOPE_CONTEXT_OR_REVISION_MISMATCH".to_owned());
        } else if text(observation, "evidence_state") != "current"
            || coverage_fraction(observation, "coverage_fraction").is_none_or(|coverage| {
                coverage < minimum_coverage.expect("minimum coverage validated")
            })
        {
            observation_state = "Unknown";
            reasons.insert("OBSERVATION_EVIDENCE_OR_COVERAGE_INSUFFICIENT".to_owned());
        } else if !owner_binding_matches(case_id, requirement, observation) {
            observation_state = "Unknown";
            reasons.insert("OBSERVATION_OWNER_RECEIPT_BINDING_INVALID".to_owned());
        } else {
            match text(requirement, "evidence_class") {
                "meter" if text(observation, "evidence_kind") != "meter_measurement" => {
                    observation_state = "Unsupported";
                    reasons.insert("OBSERVATION_EVIDENCE_CLASS_MISMATCH".to_owned());
                }
                "meter"
                    if text(observation, "calibration_state") != "current"
                        || text(observation, "instrument_id").is_empty() =>
                {
                    observation_state = "Unknown";
                    reasons.insert("METER_EVIDENCE_OR_CALIBRATION_INVALID".to_owned());
                }
                "experience"
                    if text(observation, "evidence_kind") != "consented_survey"
                        || text(observation, "consent_status") != "authorized" =>
                {
                    observation_state = "Unknown";
                    reasons.insert("EXPERIENCE_EVIDENCE_OR_CONSENT_INVALID".to_owned());
                }
                "service" if text(observation, "evidence_kind") != "service_receipt" => {
                    observation_state = "Unsupported";
                    reasons.insert("OBSERVATION_EVIDENCE_CLASS_MISMATCH".to_owned());
                }
                _ => {}
            }
        }

        if observation_state == "Current" {
            let Some(value) = number(observation, "value") else {
                observation_state = "Unsupported";
                reasons.insert("OBSERVATION_VALUE_INVALID".to_owned());
                metric_results.push(json!({
                    "metric_id": metric_id,
                    "observation_state": observation_state,
                    "threshold_state": "Unknown",
                    "observed_value": null,
                    "unit": text(requirement, "unit"),
                    "static_gap_state": "Unknown",
                    "static_gap_value": null,
                    "dynamic_gap_state": "Unknown",
                    "dynamic_gap_value": null,
                    "reason_codes": reasons.into_iter().collect::<Vec<_>>()
                }));
                continue;
            };
            observed_value = Some(value);
            let threshold = number(requirement, "threshold").expect("threshold validated");
            threshold_state = match text(requirement, "direction") {
                "le" if value <= threshold => "Pass",
                "le" => "Fail",
                "ge" if value >= threshold => "Pass",
                "ge" => "Fail",
                _ => "Unknown",
            };

            let prediction = array(input, "design_predictions")
                .unwrap_or(&[])
                .iter()
                .find(|prediction| text(prediction, "metric_id") == metric_id);
            let (state, gap, reason) =
                evaluate_static_prediction(observation, prediction, requirement, scope);
            static_gap_state = state;
            static_gap_value = gap;
            if let Some(reason) = reason {
                reasons.insert(reason);
            }

            let model = array(input, "calibrated_models")
                .unwrap_or(&[])
                .iter()
                .find(|model| text(model, "metric_id") == metric_id);
            let (state, gap, reason) =
                evaluate_dynamic_model(observation, model, requirement, scope);
            dynamic_gap_state = state;
            dynamic_gap_value = gap;
            if let Some(reason) = reason {
                reasons.insert(reason);
            }
        }

        metric_results.push(json!({
            "metric_id": metric_id,
            "observation_state": observation_state,
            "threshold_state": threshold_state,
            "observed_value": observed_value,
            "unit": text(requirement, "unit"),
            "static_gap_state": static_gap_state,
            "static_gap_value": static_gap_value,
            "dynamic_gap_state": dynamic_gap_state,
            "dynamic_gap_value": dynamic_gap_value,
            "reason_codes": reasons.into_iter().collect::<Vec<_>>()
        }));
    }

    metric_results.sort_by(|left, right| text(left, "metric_id").cmp(text(right, "metric_id")));

    json!({
        "case_id": case_id,
        "disposition": "Evaluated",
        "reason_codes": [],
        "metric_results": metric_results,
        "measurement_and_verification_status": classify_measurement_and_verification(case_id, input, scope),
        "causal_assessment_status": classify_causal_assessment(input),
        "claim_ceiling": "synthetic outcome classification only; not measured building performance, regulatory compliance, energy savings, occupant wellbeing, or causal impact"
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-in-use-outcomes/CITY_IN_USE_OUTCOMES_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("in-use outcomes corpus must exist");
        serde_json::from_slice(&bytes).expect("in-use outcomes corpus must parse")
    }

    fn case(id: &str) -> Value {
        corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["case_id"] == id)
            .unwrap()
            .clone()
    }

    #[test]
    fn corpus_has_24_unique_synthetic_cases() {
        let corpus = corpus();
        assert_eq!(corpus["classification"], "synthetic_fixture_only");
        assert_eq!(corpus["corpus_id"], "CITY-IN-USE-OUTCOMES-CORPUS-V0.1");
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 30);
        let ids = cases
            .iter()
            .map(|item| item["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn independent_oracle_matches_every_known_answer() {
        for item in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(item);
            assert!(
                matches_expected(&actual, &item["expected"]),
                "{} mismatch\nactual: {}\nexpected: {}",
                item["case_id"],
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&item["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn energy_pass_does_not_compensate_for_failed_thermal_service() {
        let result = evaluate_case(&case("OI-002"));
        assert_eq!(
            result["metric_results"]
                .as_array()
                .unwrap()
                .iter()
                .find(|metric| metric["metric_id"] == "energy")
                .unwrap()["threshold_state"],
            "Pass"
        );
        assert_eq!(
            result["metric_results"]
                .as_array()
                .unwrap()
                .iter()
                .find(|metric| metric["metric_id"] == "operative-temp")
                .unwrap()["threshold_state"],
            "Fail"
        );
    }

    #[test]
    fn static_and_dynamic_model_gaps_remain_distinct() {
        let result = evaluate_case(&case("OI-005"));
        let metric = &result["metric_results"][0];
        assert_eq!(metric["static_gap_state"], "Unknown");
        assert_eq!(metric["dynamic_gap_state"], "Comparable");
        assert_eq!(metric["dynamic_gap_value"].as_f64(), Some(10.0));
    }

    #[test]
    fn stale_meter_calibration_is_unknown_not_zero() {
        let result = evaluate_case(&case("OI-007"));
        assert_eq!(result["metric_results"][0]["observation_state"], "Unknown");
        assert_eq!(result["metric_results"][0]["observed_value"], Value::Null);
    }

    #[test]
    fn resident_survey_and_service_receipt_use_their_own_evidence_classes() {
        let consented = evaluate_case(&case("OI-024"));
        let experience = consented["metric_results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|metric| metric["metric_id"] == "comfort-report")
            .unwrap();
        assert_eq!(experience["observation_state"], "Current");
        assert_eq!(experience["threshold_state"], "Pass");

        let service = evaluate_case(&case("OI-020"));
        assert_eq!(service["metric_results"][0]["observation_state"], "Current");
        assert_eq!(service["metric_results"][0]["threshold_state"], "Fail");
    }

    #[test]
    fn survey_cannot_satisfy_meter_requirement() {
        let result = evaluate_case(&case("OI-013"));
        assert_eq!(result["disposition"], "Evaluated");
        assert_eq!(
            result["metric_results"][0]["observation_state"],
            "Unsupported"
        );
        assert_eq!(
            result["metric_results"][0]["reason_codes"],
            serde_json::json!(["OBSERVATION_EVIDENCE_CLASS_MISMATCH"])
        );
    }

    #[test]
    fn dynamic_model_requires_disjoint_validation_data_and_applicability() {
        for id in ["OI-006", "OI-015"] {
            let result = evaluate_case(&case(id));
            assert_eq!(result["metric_results"][0]["dynamic_gap_state"], "Unknown");
        }
    }

    #[test]
    fn measurement_and_verification_eligibility_is_not_a_savings_result() {
        let result = evaluate_case(&case("OI-017"));
        assert_eq!(
            result["measurement_and_verification_status"],
            "EligibleForSavingsReview"
        );
        assert!(result.get("savings_amount").is_none());
    }

    #[test]
    fn measurement_and_verification_receipt_must_bind_the_claimed_metric() {
        let mut wrong_metric = case("OI-017");
        wrong_metric["input"]["measurement_and_verification"]["post_metric_id"] =
            serde_json::json!("comfort-report");
        assert_eq!(
            evaluate_case(&wrong_metric)["measurement_and_verification_status"],
            "Unsupported"
        );
    }

    #[test]
    fn measurement_and_verification_post_window_and_receipt_must_match_observation() {
        let mut wrong_window = case("OI-017");
        wrong_window["input"]["measurement_and_verification"]["post_window"]["start_tick"] =
            serde_json::json!(1);
        assert_eq!(
            evaluate_case(&wrong_window)["measurement_and_verification_status"],
            "Unsupported"
        );

        let mut wrong_receipt = case("OI-017");
        wrong_receipt["input"]["measurement_and_verification"]["post_receipt_id"] =
            serde_json::json!("receipt:other:energy");
        assert_eq!(
            evaluate_case(&wrong_receipt)["measurement_and_verification_status"],
            "Unsupported"
        );
    }

    #[test]
    fn measurement_and_verification_requires_non_overlapping_time_windows() {
        let mut overlap = case("OI-017");
        overlap["input"]["measurement_and_verification"]["baseline_window"]["end_tick"] =
            serde_json::json!(6);
        assert_eq!(
            evaluate_case(&overlap)["measurement_and_verification_status"],
            "Unsupported"
        );

        let mut missing = case("OI-017");
        missing["input"]["measurement_and_verification"]
            .as_object_mut()
            .unwrap()
            .remove("baseline_window");
        assert_eq!(
            evaluate_case(&missing)["measurement_and_verification_status"],
            "Unsupported"
        );
    }

    #[test]
    fn before_after_observation_does_not_establish_causality() {
        assert_eq!(
            evaluate_case(&case("OI-018"))["causal_assessment_status"],
            "Unsupported"
        );
        assert_eq!(
            evaluate_case(&case("OI-019"))["causal_assessment_status"],
            "EligibleForIndependentCausalReview"
        );
    }

    #[test]
    fn scope_and_revision_mismatches_are_not_comparable() {
        for id in [
            "OI-003", "OI-004", "OI-010", "OI-011", "OI-021", "OI-022", "OI-023",
        ] {
            assert_eq!(
                evaluate_case(&case(id))["metric_results"][0]["observation_state"],
                "NotComparable"
            );
        }
    }

    #[test]
    fn incompatible_units_and_duplicate_observations_fail_closed() {
        let incompatible_unit = evaluate_case(&case("OI-008"));
        assert_eq!(incompatible_unit["disposition"], "Evaluated");
        assert_eq!(
            incompatible_unit["metric_results"][0]["observation_state"],
            "Unsupported"
        );
        assert_eq!(
            incompatible_unit["metric_results"][0]["reason_codes"],
            serde_json::json!(["OBSERVATION_UNIT_MISMATCH"])
        );
        assert_eq!(evaluate_case(&case("OI-014"))["disposition"], "Unsupported");
    }

    #[test]
    fn causal_and_savings_claims_need_declared_evidence() {
        assert_eq!(
            evaluate_case(&case("OI-016"))["measurement_and_verification_status"],
            "Unsupported"
        );
        assert_eq!(
            evaluate_case(&case("OI-019"))["causal_assessment_status"],
            "EligibleForIndependentCausalReview"
        );
    }

    #[test]
    fn dynamic_model_requires_observation_level_disjoint_validation() {
        let mut item = case("OI-005");
        item["input"]["calibrated_models"][0]["validation_observation_ids"] =
            serde_json::json!(["TRAIN-001"]);
        let result = evaluate_case(&item);
        assert_eq!(result["metric_results"][0]["dynamic_gap_state"], "Unknown");
    }

    #[test]
    fn dynamic_model_requires_a_pinned_partition_protocol_and_receipt() {
        let mut protocol_missing = case("OI-005");
        protocol_missing["input"]["calibrated_models"][0]
            .as_object_mut()
            .unwrap()
            .remove("validation_protocol_id");
        let result = evaluate_case(&protocol_missing);
        assert_eq!(result["metric_results"][0]["dynamic_gap_state"], "Unknown");

        let mut receipt_missing = case("OI-005");
        receipt_missing["input"]["calibrated_models"][0]
            .as_object_mut()
            .unwrap()
            .remove("partition_audit_receipt_id");
        let result = evaluate_case(&receipt_missing);
        assert_eq!(result["metric_results"][0]["dynamic_gap_state"], "Unknown");
    }

    #[test]
    fn observation_and_mv_coverage_cannot_exceed_one_hundred_percent() {
        let mut observation = case("OI-001");
        observation["input"]["observations"][0]["coverage_fraction"] = serde_json::json!(1.01);
        let result = evaluate_case(&observation);
        assert_eq!(result["metric_results"][0]["observation_state"], "Unknown");

        let mut mv = case("OI-017");
        mv["input"]["measurement_and_verification"]["coverage_fraction"] = serde_json::json!(1.01);
        let result = evaluate_case(&mv);
        assert_eq!(result["measurement_and_verification_status"], "Unsupported");
    }

    #[test]
    fn known_answer_match_rejects_omitted_or_extra_output_fields() {
        let expected = serde_json::json!({
            "case_id": "OI-X",
            "disposition": "Evaluated",
            "reason_codes": []
        });
        let omitted = serde_json::json!({
            "case_id": "OI-X",
            "reason_codes": []
        });
        let extra = serde_json::json!({
            "case_id": "OI-X",
            "disposition": "Evaluated",
            "reason_codes": [],
            "unreviewed_field": true
        });
        assert!(!matches_expected(&omitted, &expected));
        assert!(!matches_expected(&extra, &expected));
        assert!(matches_expected(&expected, &expected));
    }

    #[test]
    fn input_permutation_does_not_change_canonical_metric_results() {
        let mut item = case("OI-024");
        let before = evaluate_case(&item);
        item["input"]["requirements"]
            .as_array_mut()
            .unwrap()
            .reverse();
        item["input"]["observations"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let after = evaluate_case(&item);
        assert_eq!(before, after);
    }
}
