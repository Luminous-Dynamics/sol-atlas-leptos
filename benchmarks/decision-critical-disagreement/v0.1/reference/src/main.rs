use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    process::ExitCode,
};

const EPSILON: f64 = 1e-9;

#[derive(Clone)]
struct Unit {
    id: String,
    weight: f64,
    known: bool,
}

#[derive(Clone)]
struct UnitEvidence {
    prediction: Option<f64>,
    losses: BTreeMap<String, f64>,
}

fn required<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{path}.{key} is required"))
}

fn string(value: &Value, path: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{path} must be a string"))
}

fn number(value: &Value, path: &str) -> Result<f64, String> {
    let parsed = value
        .as_f64()
        .ok_or_else(|| format!("{path} must be a finite number"))?;
    if !parsed.is_finite() {
        return Err(format!("{path} must be finite"));
    }
    Ok(parsed)
}

fn unique(values: &[String], path: &str) -> Result<(), String> {
    let set: BTreeSet<&String> = values.iter().collect();
    if set.len() != values.len() {
        return Err(format!("{path} contains duplicate identifiers"));
    }
    Ok(())
}

fn optimal_actions(losses: &BTreeMap<String, f64>) -> Vec<String> {
    let minimum = losses.values().copied().fold(f64::INFINITY, f64::min);
    losses
        .iter()
        .filter(|(_, loss)| **loss <= minimum + EPSILON)
        .map(|(action, _)| action.clone())
        .collect()
}

fn evaluate_case(case: &Value) -> Result<Value, String> {
    let case_id = string(required(case, "caseId", "case")?, "case.caseId")?;

    let mut actions = required(case, "actions", "case")?
        .as_array()
        .ok_or_else(|| "case.actions must be an array".to_string())?
        .iter()
        .enumerate()
        .map(|(index, value)| string(value, &format!("case.actions[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    if actions.len() < 2 {
        return Err("case.actions must declare at least two actions".into());
    }
    unique(&actions, "case.actions")?;
    actions.sort();

    let support = required(case, "supportUnits", "case")?
        .as_array()
        .ok_or_else(|| "case.supportUnits must be an array".to_string())?;
    if support.is_empty() {
        return Err("case.supportUnits must not be empty".into());
    }

    let mut units = Vec::with_capacity(support.len());
    let mut total_support_weight = 0.0;
    let mut known_support_weight = 0.0;
    for (index, raw) in support.iter().enumerate() {
        let path = format!("case.supportUnits[{index}]");
        let id = string(required(raw, "unitId", &path)?, &format!("{path}.unitId"))?;
        let weight = number(required(raw, "weight", &path)?, &format!("{path}.weight"))?;
        if weight <= 0.0 {
            return Err(format!("{path}.weight must be positive"));
        }
        let status = string(required(raw, "status", &path)?, &format!("{path}.status"))?;
        let known = match status.as_str() {
            "known" => true,
            "unknown" => false,
            _ => return Err(format!("{path}.status must be known or unknown")),
        };
        if known {
            known_support_weight += weight;
        }
        total_support_weight += weight;
        units.push(Unit { id, weight, known });
    }
    let unit_ids: Vec<String> = units.iter().map(|unit| unit.id.clone()).collect();
    unique(&unit_ids, "case.supportUnits.unitId")?;
    units.sort_by(|left, right| left.id.cmp(&right.id));

    let raw_hypotheses = required(case, "hypotheses", "case")?
        .as_array()
        .ok_or_else(|| "case.hypotheses must be an array".to_string())?;
    if raw_hypotheses.len() < 2 {
        return Err("case.hypotheses must contain at least two alternatives".into());
    }

    let known_unit_ids: BTreeSet<String> = units
        .iter()
        .filter(|unit| unit.known)
        .map(|unit| unit.id.clone())
        .collect();
    let all_unit_ids: BTreeSet<String> = units.iter().map(|unit| unit.id.clone()).collect();

    let mut hypotheses = BTreeMap::<String, BTreeMap<String, UnitEvidence>>::new();
    for (index, raw) in raw_hypotheses.iter().enumerate() {
        let path = format!("case.hypotheses[{index}]");
        let id = string(required(raw, "hypothesisId", &path)?, &format!("{path}.hypothesisId"))?;
        let evidence_object = required(raw, "unitEvidence", &path)?
            .as_object()
            .ok_or_else(|| format!("{path}.unitEvidence must be an object"))?;

        let mut evidence = BTreeMap::new();
        for (unit_id, raw_evidence) in evidence_object {
            if !all_unit_ids.contains(unit_id) {
                return Err(format!("{path}.unitEvidence references unknown support unit {unit_id}"));
            }
            if !known_unit_ids.contains(unit_id) {
                return Err(format!(
                    "{path}.unitEvidence supplies data for unknown support unit {unit_id}"
                ));
            }
            let evidence_path = format!("{path}.unitEvidence.{unit_id}");
            let raw_losses = required(raw_evidence, "actionLosses", &evidence_path)?
                .as_object()
                .ok_or_else(|| format!("{evidence_path}.actionLosses must be an object"))?;
            if raw_losses.len() != actions.len() {
                return Err(format!(
                    "{evidence_path}.actionLosses must contain exactly the declared actions"
                ));
            }
            let mut losses = BTreeMap::new();
            for action in &actions {
                let loss = number(
                    required(
                        &Value::Object(raw_losses.clone()),
                        action,
                        &format!("{evidence_path}.actionLosses"),
                    )?,
                    &format!("{evidence_path}.actionLosses.{action}"),
                )?;
                if loss < 0.0 {
                    return Err(format!("{evidence_path}.actionLosses.{action} must be non-negative"));
                }
                losses.insert(action.clone(), loss);
            }
            let prediction = match raw_evidence.get("prediction") {
                Some(value) if !value.is_null() => {
                    Some(number(value, &format!("{evidence_path}.prediction"))?)
                }
                _ => None,
            };
            evidence.insert(unit_id.clone(), UnitEvidence { prediction, losses });
        }
        if hypotheses.insert(id.clone(), evidence).is_some() {
            return Err(format!("duplicate hypothesisId {id}"));
        }
    }

    let mut comparable_support_weight = 0.0;
    let mut decision_disagreement_weight = 0.0;
    let mut prediction_disagreement_weight = 0.0;
    let mut prediction_comparison_weight = 0.0;
    let mut weighted_prediction_range = 0.0;
    let mut maximum_cell_worst_regret = 0.0;
    let mut regret_by_hypothesis: BTreeMap<String, f64> = hypotheses
        .keys()
        .map(|id| (id.clone(), 0.0))
        .collect();
    let mut unit_reports = Vec::new();

    for unit in &units {
        if !unit.known {
            continue;
        }
        let complete = hypotheses
            .values()
            .all(|evidence| evidence.contains_key(&unit.id));
        if !complete {
            continue;
        }
        comparable_support_weight += unit.weight;

        let mut preferred = BTreeMap::<String, Vec<String>>::new();
        for (hypothesis_id, evidence) in &hypotheses {
            let unit_evidence = &evidence[&unit.id];
            preferred.insert(hypothesis_id.clone(), optimal_actions(&unit_evidence.losses));
        }
        let first_preference = preferred.values().next().cloned().unwrap_or_default();
        let disagrees = preferred.values().any(|candidate| *candidate != first_preference);
        if disagrees {
            decision_disagreement_weight += unit.weight;
        }

        let predictions: Vec<f64> = hypotheses
            .values()
            .filter_map(|evidence| evidence.get(&unit.id).and_then(|row| row.prediction))
            .collect();
        let prediction_range = if predictions.len() == hypotheses.len() {
            let minimum = predictions.iter().copied().fold(f64::INFINITY, f64::min);
            let maximum = predictions.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            prediction_comparison_weight += unit.weight;
            let range = maximum - minimum;
            weighted_prediction_range += unit.weight * range;
            if range > EPSILON {
                prediction_disagreement_weight += unit.weight;
            }
            Some(range)
        } else {
            None
        };

        let mut robust_action: Option<String> = None;
        let mut robust_score = f64::INFINITY;
        let mut worst_regret_by_hypothesis = BTreeMap::<String, f64>::new();
        for action in &actions {
            let mut maximum_regret = 0.0_f64;
            for (hypothesis_id, evidence) in &hypotheses {
                let losses = &evidence[&unit.id].losses;
                let best_loss = losses.values().copied().fold(f64::INFINITY, f64::min);
                maximum_regret = maximum_regret.max(losses[action] - best_loss);
                worst_regret_by_hypothesis.insert(hypothesis_id.clone(), losses[action] - best_loss);
            }
            if maximum_regret < robust_score - EPSILON {
                robust_score = maximum_regret;
                robust_action = Some(action.clone());
            }
        }
        let selected_action = robust_action.ok_or_else(|| format!("no robust action for {}", unit.id))?;
        maximum_cell_worst_regret = maximum_cell_worst_regret.max(robust_score);

        let mut regret_contributions = BTreeMap::<String, f64>::new();
        for (hypothesis_id, evidence) in &hypotheses {
            let losses = &evidence[&unit.id].losses;
            let best_loss = losses.values().copied().fold(f64::INFINITY, f64::min);
            let regret = losses[&selected_action] - best_loss;
            *regret_by_hypothesis.get_mut(hypothesis_id).expect("registered hypothesis") +=
                unit.weight * regret;
            regret_contributions.insert(hypothesis_id.clone(), regret);
        }

        let mut preferred_json = Map::new();
        for (hypothesis_id, selected) in preferred {
            preferred_json.insert(hypothesis_id, json!(selected));
        }
        unit_reports.push(json!({
            "unitId": unit.id,
            "weight": unit.weight,
            "modelOptimalActions": preferred_json,
            "decisionDisagreement": disagrees,
            "minimaxRegretAction": selected_action,
            "worstCaseRegretAtUnit": robust_score,
            "regretByHypothesis": regret_contributions,
            "pairwisePredictionRange": prediction_range
        }));
    }

    let unqualified_support_weight = (total_support_weight - comparable_support_weight).max(0.0);
    let support_status = if comparable_support_weight <= EPSILON {
        "NO_COMMON_SUPPORT"
    } else if (comparable_support_weight - total_support_weight).abs() <= EPSILON {
        "COMPLETE_SUPPORT"
    } else {
        "PARTIAL_OR_UNKNOWN_SUPPORT"
    };
    let worst_case_weighted_regret = regret_by_hypothesis.values().copied().fold(0.0_f64, f64::max);

    Ok(json!({
        "caseId": case_id,
        "supportStatus": support_status,
        "totalSupportWeight": total_support_weight,
        "declaredKnownSupportWeight": known_support_weight,
        "comparableSupportWeight": comparable_support_weight,
        "unqualifiedSupportWeight": unqualified_support_weight,
        "decisionDisagreementWeight": decision_disagreement_weight,
        "decisionDisagreementFractionOfComparableSupport": if comparable_support_weight > EPSILON {
            Some(decision_disagreement_weight / comparable_support_weight)
        } else {
            None
        },
        "predictionDiagnostic": {
            "comparisonSupportWeight": prediction_comparison_weight,
            "positiveRangeSupportWeight": prediction_disagreement_weight,
            "weightedMeanPairwiseRange": if prediction_comparison_weight > EPSILON {
                Some(weighted_prediction_range / prediction_comparison_weight)
            } else {
                None
            }
        },
        "minimaxRegret": {
            "worstCaseWeightedRegret": worst_case_weighted_regret,
            "regretByHypothesis": regret_by_hypothesis,
            "maximumCellWorstCaseRegret": maximum_cell_worst_regret
        },
        "units": unit_reports
    }))
}

fn evaluate_document(document: &Value) -> Result<Value, String> {
    let benchmark_id = string(required(document, "benchmarkId", "document")?, "document.benchmarkId")?;
    if benchmark_id != "sol-atlas-decision-critical-disagreement" {
        return Err("unsupported benchmarkId".into());
    }
    let cases = required(document, "cases", "document")?
        .as_array()
        .ok_or_else(|| "document.cases must be an array".to_string())?;
    if cases.is_empty() {
        return Err("document.cases must not be empty".into());
    }
    let mut case_ids = Vec::new();
    let mut results = Vec::new();
    for case in cases {
        let result = evaluate_case(case)?;
        case_ids.push(string(required(case, "caseId", "case")?, "case.caseId")?);
        results.push(result);
    }
    unique(&case_ids, "document.cases.caseId")?;
    results.sort_by(|left, right| {
        left["caseId"].as_str().cmp(&right["caseId"].as_str())
    });
    Ok(json!({
        "benchmarkId": benchmark_id,
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "claimCeiling": "SYNTHETIC_FINITE_ACTION_LOSS_ORACLE_ONLY",
        "hypothesisProbabilityAssumption": "none; hypotheses are an unweighted admissible set",
        "unknownSupportSemantics": "excluded from comparison and regret; never coerced to zero or safe",
        "caseCount": results.len(),
        "results": results
    }))
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/disagreement-cases-v0.1.json".to_string());
    let input = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {path}: {error}"))?;
    let document: Value =
        serde_json::from_str(&input).map_err(|error| format!("invalid JSON in {path}: {error}"))?;
    let report = evaluate_document(&document)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| format!("cannot serialize report: {error}"))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("decision-critical-disagreement: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../fixtures/disagreement-cases-v0.1.json");

    fn fixture() -> Value {
        serde_json::from_str(FIXTURE).expect("fixture must parse")
    }

    fn result<'a>(report: &'a Value, case_id: &str) -> &'a Value {
        report["results"]
            .as_array()
            .expect("results array")
            .iter()
            .find(|row| row["caseId"] == case_id)
            .expect("case result exists")
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn equal_scalar_predictions_can_hide_decision_disagreement() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        let row = result(&report, "same-prediction-different-action");
        close(row["decisionDisagreementWeight"].as_f64().unwrap(), 1.0);
        close(
            row["predictionDiagnostic"]["positiveRangeSupportWeight"]
                .as_f64()
                .unwrap(),
            0.0,
        );
    }

    #[test]
    fn large_prediction_difference_can_be_decision_irrelevant() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        let row = result(&report, "different-prediction-same-action");
        close(row["decisionDisagreementWeight"].as_f64().unwrap(), 0.0);
        close(
            row["predictionDiagnostic"]["positiveRangeSupportWeight"]
                .as_f64()
                .unwrap(),
            1.0,
        );
    }

    #[test]
    fn low_area_critical_corridor_retains_high_consequence_regret() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        let row = result(&report, "narrow-critical-corridor");
        close(row["decisionDisagreementWeight"].as_f64().unwrap(), 0.05);
        close(
            row["minimaxRegret"]["maximumCellWorstCaseRegret"]
                .as_f64()
                .unwrap(),
            1000.0,
        );
        close(
            row["minimaxRegret"]["worstCaseWeightedRegret"]
                .as_f64()
                .unwrap(),
            50.0,
        );
    }

    #[test]
    fn partial_or_unknown_support_is_not_imputed() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        let row = result(&report, "partial-support");
        assert_eq!(row["supportStatus"], "PARTIAL_OR_UNKNOWN_SUPPORT");
        close(row["comparableSupportWeight"].as_f64().unwrap(), 0.8);
        close(row["unqualifiedSupportWeight"].as_f64().unwrap(), 0.3);
    }

    #[test]
    fn no_common_support_does_not_emit_a_zero_disagreement_claim() {
        let mut document = fixture();
        document["cases"][0]["hypotheses"][1]["unitEvidence"] = json!({});
        let row = evaluate_case(&document["cases"][0]).expect("case remains inspectable");
        assert_eq!(row["supportStatus"], "NO_COMMON_SUPPORT");
        assert!(row["decisionDisagreementFractionOfComparableSupport"].is_null());
    }

    #[test]
    fn model_and_support_order_do_not_change_results() {
        let document = fixture();
        let baseline = evaluate_document(&document).expect("baseline evaluates");
        let mut reordered = document.clone();
        for case in reordered["cases"].as_array_mut().unwrap() {
            case["actions"].as_array_mut().unwrap().reverse();
            case["supportUnits"].as_array_mut().unwrap().reverse();
            case["hypotheses"].as_array_mut().unwrap().reverse();
        }
        assert_eq!(
            baseline,
            evaluate_document(&reordered).expect("reordered fixture evaluates")
        );
    }

    #[test]
    fn rejects_missing_action_loss() {
        let mut document = fixture();
        document["cases"][0]["hypotheses"][0]["unitEvidence"]["reach-a"]["actionLosses"]
            .as_object_mut()
            .unwrap()
            .remove("build");
        assert!(evaluate_document(&document).is_err());
    }

    #[test]
    fn rejects_evidence_for_unknown_support() {
        let mut document = fixture();
        document["cases"][3]["hypotheses"][0]["unitEvidence"]["unknown-area"] =
            json!({"actionLosses":{"protect":0.0,"build":1.0}});
        assert!(evaluate_document(&document).is_err());
    }

    #[test]
    fn rejects_negative_loss() {
        let mut document = fixture();
        document["cases"][0]["hypotheses"][0]["unitEvidence"]["reach-a"]["actionLosses"]["protect"] =
            json!(-1.0);
        assert!(evaluate_document(&document).is_err());
    }
}
