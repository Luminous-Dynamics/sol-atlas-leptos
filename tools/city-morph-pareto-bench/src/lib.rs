//! Bounded reference oracle for evidence-qualified Pareto comparison.
//!
//! Hard constraints are non-compensable. Objectives stay in their declared
//! units. Robust dominance requires separation of uncertainty intervals.
//! Unknown or infeasible alternatives remain visible but cannot enter the
//! feasible Pareto frontier. This is not a planning decision or social choice.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Objective {
    metric_id: String,
    direction: String,
}

#[derive(Clone, Debug)]
struct Constraint {
    metric_id: String,
    comparator: String,
    threshold: f64,
}

fn array<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice)
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn finite_number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64).filter(|number| number.is_finite())
}

fn allowed_evidence_state(value: &str) -> bool {
    matches!(value, "current" | "unknown" | "missing" | "stale" | "revoked")
}

fn empty_unsupported(case_id: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "disposition": "Unsupported",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "alternative_statuses": [],
        "feasible_alternative_ids": [],
        "infeasible_alternative_ids": [],
        "unknown_alternative_ids": [],
        "dominance_relations": [],
        "pareto_frontier": [],
        "incomparable_pairs": []
    })
}

fn interval(value: &Value) -> Option<(f64, f64)> {
    let lower = finite_number(value, "lower")?;
    let upper = finite_number(value, "upper")?;
    (lower <= upper).then_some((lower, upper))
}

fn robustly_dominates(
    candidate: &BTreeMap<String, Value>,
    other: &BTreeMap<String, Value>,
    objectives: &[Objective],
) -> bool {
    let mut strictly_better = false;

    for objective in objectives {
        let Some(candidate_metric) = candidate.get(&objective.metric_id) else {
            return false;
        };
        let Some(other_metric) = other.get(&objective.metric_id) else {
            return false;
        };
        let Some((candidate_lower, candidate_upper)) = interval(candidate_metric) else {
            return false;
        };
        let Some((other_lower, other_upper)) = interval(other_metric) else {
            return false;
        };

        match objective.direction.as_str() {
            "minimize" => {
                if candidate_upper > other_lower {
                    return false;
                }
                if candidate_upper < other_lower {
                    strictly_better = true;
                }
            }
            "maximize" => {
                if candidate_lower < other_upper {
                    return false;
                }
                if candidate_lower > other_upper {
                    strictly_better = true;
                }
            }
            _ => return false,
        }
    }

    strictly_better
}

/// Recompute one comparison case from its input only. The expected field is
/// never read by this function and cannot influence feasibility or dominance.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = string(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut unsupported = BTreeSet::<String>::new();

    let scope_digest = string(input, "scope_digest");
    if scope_digest.is_empty() {
        unsupported.insert("EVALUATION_SCOPE_DIGEST_MISSING".to_owned());
    }

    let Some(objective_values) = array(input, "objectives") else {
        unsupported.insert("OBJECTIVE_DEFINITIONS_MISSING".to_owned());
        return empty_unsupported(case_id, unsupported);
    };
    if objective_values.is_empty() {
        unsupported.insert("OBJECTIVE_DEFINITIONS_MISSING".to_owned());
    }

    let Some(constraint_values) = array(input, "constraints") else {
        unsupported.insert("CONSTRAINT_DEFINITIONS_MISSING".to_owned());
        return empty_unsupported(case_id, unsupported);
    };

    let Some(alternatives) = array(input, "alternatives") else {
        unsupported.insert("ALTERNATIVES_MISSING".to_owned());
        return empty_unsupported(case_id, unsupported);
    };
    if alternatives.is_empty() {
        unsupported.insert("ALTERNATIVES_EMPTY".to_owned());
    }

    let mut metric_units = BTreeMap::<String, String>::new();
    let mut objectives = Vec::<Objective>::new();
    let mut objective_ids = BTreeSet::<String>::new();

    for objective in objective_values {
        let metric_id = string(objective, "metric_id");
        let unit = string(objective, "unit");
        let direction = string(objective, "direction");
        let definition_digest = string(objective, "definition_digest");

        if metric_id.is_empty()
            || unit.is_empty()
            || definition_digest.is_empty()
            || !matches!(direction, "minimize" | "maximize")
        {
            unsupported.insert("OBJECTIVE_DEFINITION_INVALID".to_owned());
            continue;
        }
        if !objective_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_OBJECTIVE_METRIC_ID".to_owned());
            continue;
        }
        if metric_units
            .insert(metric_id.to_owned(), unit.to_owned())
            .is_some_and(|prior| prior != unit)
        {
            unsupported.insert("METRIC_DEFINITION_UNIT_CONFLICT".to_owned());
        }
        objectives.push(Objective {
            metric_id: metric_id.to_owned(),
            direction: direction.to_owned(),
        });
    }

    let mut constraints = Vec::<Constraint>::new();
    let mut constraint_ids = BTreeSet::<String>::new();

    for constraint in constraint_values {
        let metric_id = string(constraint, "metric_id");
        let unit = string(constraint, "unit");
        let comparator = string(constraint, "comparator");
        let definition_digest = string(constraint, "definition_digest");
        let Some(threshold) = finite_number(constraint, "threshold") else {
            unsupported.insert("CONSTRAINT_DEFINITION_INVALID".to_owned());
            continue;
        };

        if metric_id.is_empty()
            || unit.is_empty()
            || definition_digest.is_empty()
            || !matches!(comparator, "le" | "ge")
        {
            unsupported.insert("CONSTRAINT_DEFINITION_INVALID".to_owned());
            continue;
        }
        if !constraint_ids.insert(metric_id.to_owned()) {
            unsupported.insert("DUPLICATE_CONSTRAINT_METRIC_ID".to_owned());
            continue;
        }
        if metric_units
            .insert(metric_id.to_owned(), unit.to_owned())
            .is_some_and(|prior| prior != unit)
        {
            unsupported.insert("METRIC_DEFINITION_UNIT_CONFLICT".to_owned());
        }
        constraints.push(Constraint {
            metric_id: metric_id.to_owned(),
            comparator: comparator.to_owned(),
            threshold,
        });
    }

    let mut alternative_ids = BTreeSet::<String>::new();
    let mut alternative_metrics = BTreeMap::<String, BTreeMap<String, Value>>::new();

    for alternative in alternatives {
        let alternative_id = string(alternative, "id");
        if alternative_id.is_empty() || !alternative_ids.insert(alternative_id.to_owned()) {
            unsupported.insert("DUPLICATE_OR_MISSING_ALTERNATIVE_ID".to_owned());
            continue;
        }
        if string(alternative, "scope_digest") != scope_digest {
            unsupported.insert("ALTERNATIVE_SCOPE_MISMATCH".to_owned());
        }

        let Some(metrics) = array(alternative, "metrics") else {
            unsupported.insert("ALTERNATIVE_METRICS_MISSING".to_owned());
            continue;
        };
        let mut metric_ids = BTreeSet::<String>::new();
        let mut metric_map = BTreeMap::<String, Value>::new();

        for metric in metrics {
            let metric_id = string(metric, "metric_id");
            let unit = string(metric, "unit");
            let evidence_state = string(metric, "evidence_state");

            if metric_id.is_empty() || !metric_ids.insert(metric_id.to_owned()) {
                unsupported.insert("DUPLICATE_OR_MISSING_ALTERNATIVE_METRIC_ID".to_owned());
                continue;
            }
            let Some(expected_unit) = metric_units.get(metric_id) else {
                unsupported.insert("ALTERNATIVE_METRIC_UNDEFINED".to_owned());
                continue;
            };
            if unit != expected_unit {
                unsupported.insert("ALTERNATIVE_METRIC_UNIT_MISMATCH".to_owned());
            }
            if !allowed_evidence_state(evidence_state) {
                unsupported.insert("METRIC_EVIDENCE_STATE_UNSUPPORTED".to_owned());
            }
            if evidence_state == "current" && interval(metric).is_none() {
                unsupported.insert("METRIC_INTERVAL_INVALID".to_owned());
            }
            metric_map.insert(metric_id.to_owned(), metric.clone());
        }
        alternative_metrics.insert(alternative_id.to_owned(), metric_map);
    }

    if !unsupported.is_empty() {
        return empty_unsupported(case_id, unsupported);
    }

    let mut alternative_statuses = Vec::<Value>::new();
    let mut feasible_ids = Vec::<String>::new();
    let mut infeasible_ids = Vec::<String>::new();
    let mut unknown_ids = Vec::<String>::new();

    for alternative_id in &alternative_ids {
        let metrics = alternative_metrics
            .get(alternative_id)
            .expect("validated alternative metrics");
        let mut violated = false;
        let mut unknown = false;
        let mut reason_codes = BTreeSet::<String>::new();

        for constraint in &constraints {
            match metrics.get(&constraint.metric_id) {
                Some(metric) if string(metric, "evidence_state") == "current" => {
                    let (lower, upper) = interval(metric).expect("current intervals validated");
                    match constraint.comparator.as_str() {
                        "le" if lower > constraint.threshold => {
                            violated = true;
                            reason_codes.insert(format!(
                                "HARD_CONSTRAINT_VIOLATED:{}",
                                constraint.metric_id
                            ));
                        }
                        "le" if upper > constraint.threshold => {
                            unknown = true;
                            reason_codes.insert(format!(
                                "HARD_CONSTRAINT_STRADDLES:{}",
                                constraint.metric_id
                            ));
                        }
                        "ge" if upper < constraint.threshold => {
                            violated = true;
                            reason_codes.insert(format!(
                                "HARD_CONSTRAINT_VIOLATED:{}",
                                constraint.metric_id
                            ));
                        }
                        "ge" if lower < constraint.threshold => {
                            unknown = true;
                            reason_codes.insert(format!(
                                "HARD_CONSTRAINT_STRADDLES:{}",
                                constraint.metric_id
                            ));
                        }
                        _ => {}
                    }
                }
                _ => {
                    unknown = true;
                    reason_codes.insert(format!(
                        "CONSTRAINT_EVIDENCE_UNKNOWN:{}",
                        constraint.metric_id
                    ));
                }
            }
        }

        for objective in &objectives {
            match metrics.get(&objective.metric_id) {
                Some(metric) if string(metric, "evidence_state") == "current" => {}
                _ => {
                    unknown = true;
                    reason_codes.insert(format!(
                        "OBJECTIVE_EVIDENCE_UNKNOWN:{}",
                        objective.metric_id
                    ));
                }
            }
        }

        let status = if violated {
            infeasible_ids.push(alternative_id.clone());
            "Infeasible"
        } else if unknown {
            unknown_ids.push(alternative_id.clone());
            "Unknown"
        } else {
            feasible_ids.push(alternative_id.clone());
            "Feasible"
        };

        alternative_statuses.push(json!({
            "alternative_id": alternative_id,
            "status": status,
            "reason_codes": reason_codes.into_iter().collect::<Vec<_>>()
        }));
    }

    let mut dominance_relations = Vec::<Value>::new();
    for candidate_id in &feasible_ids {
        for other_id in &feasible_ids {
            if candidate_id == other_id {
                continue;
            }
            let candidate = alternative_metrics
                .get(candidate_id)
                .expect("feasible candidate metrics");
            let other = alternative_metrics
                .get(other_id)
                .expect("feasible other metrics");
            if robustly_dominates(candidate, other, &objectives) {
                dominance_relations.push(json!({
                    "dominator": candidate_id,
                    "dominated": other_id
                }));
            }
        }
    }

    let dominated_ids = dominance_relations
        .iter()
        .filter_map(|relation| relation.get("dominated").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let pareto_frontier = feasible_ids
        .iter()
        .filter(|alternative_id| !dominated_ids.contains(alternative_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let mut incomparable_pairs = Vec::<Value>::new();
    for left_index in 0..feasible_ids.len() {
        for right_index in (left_index + 1)..feasible_ids.len() {
            let left_id = &feasible_ids[left_index];
            let right_id = &feasible_ids[right_index];
            let left = alternative_metrics.get(left_id).expect("left metrics");
            let right = alternative_metrics.get(right_id).expect("right metrics");
            if !robustly_dominates(left, right, &objectives)
                && !robustly_dominates(right, left, &objectives)
            {
                incomparable_pairs.push(json!({
                    "alternative_a": left_id,
                    "alternative_b": right_id,
                    "reason": "TRADEOFF_OR_OVERLAPPING_UNCERTAINTY_OR_TIE"
                }));
            }
        }
    }

    json!({
        "case_id": case_id,
        "disposition": "Comparable",
        "reason_codes": [],
        "alternative_statuses": alternative_statuses,
        "feasible_alternative_ids": feasible_ids,
        "infeasible_alternative_ids": infeasible_ids,
        "unknown_alternative_ids": unknown_ids,
        "dominance_relations": dominance_relations,
        "pareto_frontier": pareto_frontier,
        "incomparable_pairs": incomparable_pairs
    })
}

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
                    .zip(right.iter())
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

/// Compare declared known-answer fields. Numeric representations compare by
/// exact value; strings, arrays and object keys remain exact.
pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected.as_object().is_some_and(|fields| {
        fields.iter().all(|(key, expected_value)| {
            actual
                .get(key)
                .is_some_and(|actual_value| json_equivalent(actual_value, expected_value))
        })
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
            .join("../../docs/research/city-morph-pareto/CITY_MORPH_PARETO_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("Pareto corpus must exist");
        serde_json::from_slice(&bytes).expect("Pareto corpus must parse")
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
        assert_eq!(corpus["corpus_id"], "CITY-MORPH-PARETO-CORPUS-V0.1");
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        let ids = cases
            .iter()
            .map(|item| item["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), cases.len());
    }

    #[test]
    fn independent_reference_oracle_matches_every_known_answer() {
        let corpus = corpus();
        for item in corpus["cases"].as_array().unwrap() {
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
    fn overlapping_intervals_do_not_create_robust_dominance() {
        let result = evaluate_case(&case("MP-007"));
        assert_eq!(result["dominance_relations"], serde_json::json!([]));
        assert_eq!(result["pareto_frontier"], serde_json::json!(["A", "B"]));
    }

    #[test]
    fn hard_constraint_violation_cannot_be_compensated() {
        let result = evaluate_case(&case("MP-015"));
        assert_eq!(result["alternative_statuses"][0]["status"], "Infeasible");
        assert_eq!(result["pareto_frontier"], serde_json::json!(["B"]));
        assert_eq!(result["infeasible_alternative_ids"], serde_json::json!(["A"]));
    }

    #[test]
    fn unknown_alternative_cannot_dominate_a_feasible_alternative() {
        let result = evaluate_case(&case("MP-019"));
        assert_eq!(result["unknown_alternative_ids"], serde_json::json!(["A"]));
        assert_eq!(result["feasible_alternative_ids"], serde_json::json!(["B"]));
        assert_eq!(result["dominance_relations"], serde_json::json!([]));
        assert_eq!(result["pareto_frontier"], serde_json::json!(["B"]));
    }

    #[test]
    fn infeasible_alternative_stays_visible_but_outside_frontier() {
        let result = evaluate_case(&case("MP-021"));
        assert_eq!(result["infeasible_alternative_ids"], serde_json::json!(["A"]));
        assert_eq!(result["pareto_frontier"], serde_json::json!(["B"]));
    }

    #[test]
    fn equal_vectors_are_ties_not_strict_dominance() {
        let result = evaluate_case(&case("MP-005"));
        assert_eq!(result["dominance_relations"], serde_json::json!([]));
        assert_eq!(result["pareto_frontier"], serde_json::json!(["A", "B"]));
        assert_eq!(result["incomparable_pairs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn weak_no_worse_and_one_strict_objective_is_dominance() {
        let result = evaluate_case(&case("MP-018"));
        assert_eq!(
            result["dominance_relations"],
            serde_json::json!([{"dominator":"A","dominated":"B"}])
        );
        assert_eq!(result["pareto_frontier"], serde_json::json!(["A"]));
    }

    #[test]
    fn incompatible_units_and_scope_are_unsupported() {
        for id in ["MP-011", "MP-012"] {
            let result = evaluate_case(&case(id));
            assert_eq!(result["disposition"], "Unsupported");
            assert_eq!(result["pareto_frontier"], serde_json::json!([]));
        }
    }

    #[test]
    fn input_permutation_does_not_change_normalized_result() {
        let mut item = case("MP-017");
        let baseline = evaluate_case(&item);
        item["input"]["objectives"].as_array_mut().unwrap().reverse();
        item["input"]["alternatives"].as_array_mut().unwrap().reverse();
        for alternative in item["input"]["alternatives"].as_array_mut().unwrap() {
            alternative["metrics"].as_array_mut().unwrap().reverse();
        }
        let permuted = evaluate_case(&item);
        assert_eq!(baseline, permuted);
    }
}
