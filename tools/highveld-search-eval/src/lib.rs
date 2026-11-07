//! Offline audit of matched multi-objective search benchmark bundles.
//!
//! The evaluator validates benchmark methodology and computes a small,
//! deterministic set of mathematical metrics. It is not a building simulator,
//! geometry engine, climate validator or engineering approval authority.

use serde_json::{Value, json};
use std::collections::HashSet;

const CLAIM_CEILING: &str = "mathematical_synthetic_metric_audit_only";

#[derive(Clone, Debug, PartialEq)]
pub struct RunReport {
    pub seed: Option<u64>,
    pub candidate_count: usize,
    pub feasible_count: usize,
    pub false_pass_count: usize,
    pub false_reject_count: usize,
    pub pareto_ids: Vec<String>,
    pub excluded: Vec<(String, Vec<&'static str>)>,
    pub hypervolume_2d: Option<f64>,
    pub high_fidelity_calls_used: u64,
    pub low_fidelity_calls: u64,
    pub wall_time_ms_used: u64,
    pub human_minutes_used: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodReport {
    pub id: String,
    pub algorithm: String,
    pub stochastic: bool,
    pub runs: Vec<RunReport>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationReport {
    pub fixture_id: String,
    pub matched_comparison: bool,
    pub exact_reference_method: Option<String>,
    pub exact_reference_front_ids: Vec<String>,
    pub objective_count: usize,
    pub methods: Vec<MethodReport>,
    pub unsupported_metrics: Vec<&'static str>,
    pub violations: Vec<&'static str>,
}

impl EvaluationReport {
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty() && self.matched_comparison
    }

    pub fn to_value(&self) -> Value {
        let methods: Vec<Value> = self
            .methods
            .iter()
            .map(|method| {
                let runs: Vec<Value> = method
                    .runs
                    .iter()
                    .map(|run| {
                        let excluded: Vec<Value> = run
                            .excluded
                            .iter()
                            .map(|(id, reasons)| {
                                json!({
                                    "candidate_id": id,
                                    "reasons": reasons,
                                })
                            })
                            .collect();
                        json!({
                            "seed": run.seed,
                            "candidate_count": run.candidate_count,
                            "feasible_count": run.feasible_count,
                            "false_pass_count": run.false_pass_count,
                            "false_reject_count": run.false_reject_count,
                            "pareto_ids": run.pareto_ids,
                            "excluded": excluded,
                            "hypervolume_2d": run.hypervolume_2d,
                            "high_fidelity_calls_used": run.high_fidelity_calls_used,
                            "low_fidelity_calls": run.low_fidelity_calls,
                            "wall_time_ms_used": run.wall_time_ms_used,
                            "human_minutes_used": run.human_minutes_used,
                        })
                    })
                    .collect();
                json!({
                    "id": method.id,
                    "algorithm": method.algorithm,
                    "stochastic": method.stochastic,
                    "run_count": method.runs.len(),
                    "runs": runs,
                })
            })
            .collect();

        json!({
            "fixture_id": self.fixture_id,
            "claim_ceiling": CLAIM_CEILING,
            "matched_comparison": self.matched_comparison,
            "exact_reference_method": self.exact_reference_method,
            "exact_reference_front_ids": self.exact_reference_front_ids,
            "objective_count": self.objective_count,
            "methods": methods,
            "unsupported_metrics": self.unsupported_metrics,
            "violations": self.violations,
            "result": if self.is_valid() {
                "benchmark_bundle_valid_mathematical_fixture_only"
            } else {
                "invalid"
            },
            "claim_limit": "no_building_performance_or_sota_superiority_claim",
        })
    }
}

#[derive(Clone, Debug)]
struct Objective {
    id: String,
    direction: String,
    lower: f64,
    upper: f64,
}

#[derive(Clone, Debug)]
struct CandidateEvaluation {
    id: String,
    feasible: bool,
    normalized: Option<Vec<f64>>,
    exclusion_reasons: Vec<&'static str>,
    declared_feasible: bool,
    declaration_valid: bool,
}

fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn get_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn get_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn is_sha256(value: Option<&str>) -> bool {
    value.is_some_and(|s| s.len() == 64 && s.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn push_unique(violations: &mut Vec<&'static str>, id: &'static str) {
    if !violations.contains(&id) {
        violations.push(id);
    }
}

fn objectives_from(bundle: &Value, violations: &mut Vec<&'static str>) -> Vec<Objective> {
    let mut out = Vec::new();
    let Some(items) = bundle.get("objectives").and_then(Value::as_array) else {
        push_unique(violations, "OBJECTIVE_SCHEMA_MISSING");
        return out;
    };
    let mut ids = HashSet::new();
    for item in items {
        let Some(id) = get_str(item, "id") else {
            push_unique(violations, "OBJECTIVE_ID_MISSING");
            continue;
        };
        if id.is_empty() || !ids.insert(id.to_owned()) {
            push_unique(violations, "OBJECTIVE_ID_DUPLICATED");
            continue;
        }
        let direction = get_str(item, "direction").unwrap_or_default();
        if direction != "minimize" && direction != "maximize" {
            push_unique(violations, "OBJECTIVE_DIRECTION_INVALID");
            continue;
        }
        let lower = item.get("normalization_lower").and_then(Value::as_f64);
        let upper = item.get("normalization_upper").and_then(Value::as_f64);
        let unit = get_str(item, "unit");
        match (lower, upper) {
            (Some(lo), Some(hi))
                if lo.is_finite()
                    && hi.is_finite()
                    && lo < hi
                    && unit.is_some_and(|u| !u.trim().is_empty()) =>
            {
                out.push(Objective {
                    id: id.to_owned(),
                    direction: direction.to_owned(),
                    lower: lo,
                    upper: hi,
                });
            }
            _ => push_unique(violations, "OBJECTIVE_NORMALIZATION_INVALID"),
        }
    }
    if out.is_empty() {
        push_unique(violations, "OBJECTIVE_SCHEMA_EMPTY");
    }
    out
}

fn normalize_objectives(
    candidate: &Value,
    objectives: &[Objective],
) -> Result<Vec<f64>, &'static str> {
    let values = candidate
        .get("objectives")
        .and_then(Value::as_object)
        .ok_or("OBJECTIVES_MISSING")?;
    let mut normalized = Vec::with_capacity(objectives.len());
    for objective in objectives {
        let value = values
            .get(&objective.id)
            .and_then(Value::as_f64)
            .filter(|number| number.is_finite())
            .ok_or("OBJECTIVE_VALUE_MISSING_OR_NONFINITE")?;
        let span = objective.upper - objective.lower;
        let score = if objective.direction == "minimize" {
            (value - objective.lower) / span
        } else {
            (objective.upper - value) / span
        };
        if !score.is_finite() {
            return Err("OBJECTIVE_NORMALIZATION_NONFINITE");
        }
        normalized.push(score);
    }
    Ok(normalized)
}

fn required_gate_ids(bundle: &Value, violations: &mut Vec<&'static str>) -> Vec<String> {
    let Some(items) = bundle.get("hard_gates").and_then(Value::as_array) else {
        push_unique(violations, "HARD_GATE_SCHEMA_MISSING");
        return Vec::new();
    };
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for item in items {
        let Some(id) = get_str(item, "id") else {
            push_unique(violations, "HARD_GATE_ID_MISSING");
            continue;
        };
        if id.trim().is_empty() || !seen.insert(id.to_owned()) {
            push_unique(violations, "HARD_GATE_ID_DUPLICATED");
            continue;
        }
        if get_str(item, "authority_scope").is_none_or(str::is_empty) {
            push_unique(violations, "HARD_GATE_AUTHORITY_SCOPE_MISSING");
        }
        ids.push(id.to_owned());
    }
    if ids.is_empty() {
        push_unique(violations, "HARD_GATE_SCHEMA_EMPTY");
    }
    ids
}

fn gate_evidence_is_complete(candidate: &Value, gate_ids: &[String]) -> bool {
    let Some(evidence) = candidate
        .get("hard_gate_evidence_sha256")
        .and_then(Value::as_object)
    else {
        return false;
    };
    evidence.len() == gate_ids.len()
        && gate_ids
            .iter()
            .all(|gate_id| is_sha256(evidence.get(gate_id).and_then(Value::as_str)))
}

fn evaluate_candidate(
    candidate: &Value,
    objectives: &[Objective],
    gate_ids: &[String],
) -> CandidateEvaluation {
    let id = get_str(candidate, "id")
        .unwrap_or("UNKNOWN_CANDIDATE_ID")
        .to_owned();
    let declaration = get_bool(candidate, "reported_feasible");
    let declaration_valid = declaration.is_some();
    let declared_feasible = declaration.unwrap_or(false);
    let mut exclusion_reasons = Vec::new();

    if !declaration_valid {
        exclusion_reasons.push("FEASIBILITY_DECLARATION_MISSING_OR_INVALID");
    }
    if get_str(candidate, "id").is_none_or(str::is_empty)
        || !is_sha256(get_str(candidate, "input_sha256"))
    {
        exclusion_reasons.push("IDENTITY_OR_INPUT_DIGEST_INVALID");
    }
    if !is_sha256(get_str(candidate, "candidate_revision_sha256")) {
        exclusion_reasons.push("CANDIDATE_REVISION_DIGEST_INVALID");
    }

    match get_str(candidate, "dependency_status") {
        Some("current") => {}
        Some("stale") => exclusion_reasons.push("DEPENDENCY_STALE"),
        _ => exclusion_reasons.push("DEPENDENCY_STATUS_UNKNOWN"),
    }

    let verification = candidate.get("verification");
    match verification.and_then(|v| get_str(v, "kind")) {
        Some("high_fidelity") => {}
        Some("surrogate_only") => exclusion_reasons.push("SURROGATE_ONLY_NOT_FINAL_EVIDENCE"),
        _ => exclusion_reasons.push("VERIFICATION_KIND_UNKNOWN"),
    }
    if verification.and_then(|v| get_bool(v, "converged")) != Some(true) {
        exclusion_reasons.push("SOLVER_NOT_CONFIRMED_CONVERGED");
    }
    if verification.and_then(|v| get_bool(v, "required_inputs_complete")) != Some(true) {
        exclusion_reasons.push("REQUIRED_SOLVER_INPUTS_INCOMPLETE");
    }
    if verification
        .and_then(|v| get_str(v, "validation_scope"))
        .is_none_or(str::is_empty)
    {
        exclusion_reasons.push("SOLVER_VALIDATION_SCOPE_MISSING");
    }
    if !is_sha256(verification.and_then(|value| get_str(value, "result_sha256"))) {
        exclusion_reasons.push("SOLVER_RESULT_DIGEST_INVALID");
    }
    let warnings = verification
        .and_then(|v| v.get("warnings"))
        .and_then(Value::as_array);
    match warnings {
        None => exclusion_reasons.push("SOLVER_WARNING_LIST_MISSING_OR_INVALID"),
        Some(items)
            if !items.is_empty()
                && verification.and_then(|v| get_bool(v, "warnings_acknowledged"))
                    != Some(true) =>
        {
            exclusion_reasons.push("SOLVER_WARNINGS_UNACKNOWLEDGED");
        }
        _ => {}
    }

    let gates = candidate.get("hard_gates").and_then(Value::as_object);
    if gates.is_none() {
        exclusion_reasons.push("HARD_GATE_RESULTS_MISSING");
    }
    for gate_id in gate_ids {
        match gates
            .and_then(|map| map.get(gate_id))
            .and_then(Value::as_str)
        {
            Some("passed") => {}
            Some("failed") => exclusion_reasons.push("HARD_GATE_FAILED"),
            Some("unknown") => exclusion_reasons.push("HARD_GATE_UNKNOWN"),
            _ => exclusion_reasons.push("HARD_GATE_STATUS_MISSING_OR_INVALID"),
        }
    }
    if gates.is_some_and(|map| map.keys().any(|id| !gate_ids.contains(id))) {
        exclusion_reasons.push("UNDECLARED_HARD_GATE_RESULT");
    }
    if !gate_evidence_is_complete(candidate, gate_ids) {
        exclusion_reasons.push("HARD_GATE_EVIDENCE_REFERENCE_MISSING_OR_INVALID");
    }

    let normalized = match normalize_objectives(candidate, objectives) {
        Ok(values) => Some(values),
        Err(reason) => {
            exclusion_reasons.push(reason);
            None
        }
    };

    let feasible = exclusion_reasons.is_empty();
    CandidateEvaluation {
        id,
        feasible,
        normalized,
        exclusion_reasons,
        declared_feasible,
        declaration_valid,
    }
}

fn dominates(a: &[f64], b: &[f64]) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    let no_worse = a.iter().zip(b).all(|(left, right)| *left <= *right + 1e-12);
    let strictly_better = a.iter().zip(b).any(|(left, right)| *left < *right - 1e-12);
    no_worse && strictly_better
}

fn pareto_indices(points: &[Vec<f64>]) -> Vec<usize> {
    let mut front = Vec::new();
    for (index, point) in points.iter().enumerate() {
        let is_dominated = points
            .iter()
            .enumerate()
            .any(|(other_index, other)| other_index != index && dominates(other, point));
        if !is_dominated {
            front.push(index);
        }
    }
    front
}

/// Exact two-dimensional hypervolume for minimization points and a fixed
/// reference point. For other dimensions the metric is deliberately unavailable.
pub fn hypervolume_2d(points: &[[f64; 2]], reference: [f64; 2]) -> Option<f64> {
    if !reference.iter().all(|value| value.is_finite()) {
        return None;
    }
    if points.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }

    let mut points: Vec<[f64; 2]> = points
        .iter()
        .copied()
        .filter(|point| point[0] < reference[0] && point[1] < reference[1])
        .collect();
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then_with(|| a[1].total_cmp(&b[1])));

    let mut best_y = reference[1];
    let mut area = 0.0;
    for point in points {
        if point[1] < best_y {
            area += (reference[0] - point[0]) * (best_y - point[1]);
            best_y = point[1];
        }
    }
    area.is_finite().then_some(area)
}

fn hypervolume_for_front(points: &[Vec<f64>], reference: &[f64]) -> Option<f64> {
    if points.is_empty() || points.first().is_none_or(|point| point.len() != 2) {
        return None;
    }
    if reference.len() != 2 {
        return None;
    }
    let fixed: Vec<[f64; 2]> = points.iter().map(|point| [point[0], point[1]]).collect();
    hypervolume_2d(&fixed, [reference[0], reference[1]])
}

fn u64_field(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn validate_run_accounting(
    run: &Value,
    high_fidelity_budget: u64,
    low_fidelity_budget: u64,
    wall_time_ms_budget: u64,
    human_minutes_budget: u64,
    candidate_budget: u64,
    violations: &mut Vec<&'static str>,
) -> (u64, u64, u64, u64) {
    let Some(calls) = run.get("high_fidelity_calls") else {
        push_unique(violations, "HIGH_FIDELITY_CALL_ACCOUNTING_MISSING");
        return (0, 0, 0, 0);
    };
    let fields = ["training", "tuning", "validation", "final"];
    let mut total = 0u64;
    for field in fields {
        match u64_field(calls, field) {
            Some(value) => match total.checked_add(value) {
                Some(sum) => total = sum,
                None => push_unique(violations, "HIGH_FIDELITY_CALL_ACCOUNTING_OVERFLOW"),
            },
            None => push_unique(violations, "HIGH_FIDELITY_CALL_ACCOUNTING_INCOMPLETE"),
        }
    }
    let final_calls = u64_field(calls, "final").unwrap_or(0);
    let high_fidelity_candidate_count = run
        .get("candidates")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|candidate| {
                    candidate
                        .get("verification")
                        .and_then(|value| get_str(value, "kind"))
                        == Some("high_fidelity")
                })
                .count() as u64
        })
        .unwrap_or(0);
    if final_calls < high_fidelity_candidate_count {
        push_unique(violations, "FINAL_CALLS_UNDERREPORTED");
    }

    let low = u64_field(run, "low_fidelity_calls").unwrap_or_else(|| {
        push_unique(violations, "LOW_FIDELITY_CALL_COUNT_MISSING");
        0
    });
    if low > low_fidelity_budget {
        push_unique(violations, "LOW_FIDELITY_BUDGET_EXCEEDED");
    }
    let wall_time = u64_field(run, "wall_time_ms_used").unwrap_or_else(|| {
        push_unique(violations, "WALL_TIME_ACCOUNTING_MISSING");
        0
    });
    if wall_time > wall_time_ms_budget {
        push_unique(violations, "WALL_TIME_BUDGET_EXCEEDED");
    }
    let human = u64_field(run, "human_minutes_used").unwrap_or_else(|| {
        push_unique(violations, "HUMAN_TIME_ACCOUNTING_MISSING");
        0
    });
    let candidate_count = run
        .get("candidates")
        .and_then(Value::as_array)
        .map_or(0, |items| items.len()) as u64;

    if total > high_fidelity_budget {
        push_unique(violations, "HIGH_FIDELITY_BUDGET_EXCEEDED");
    }
    if human > human_minutes_budget {
        push_unique(violations, "HUMAN_TIME_BUDGET_EXCEEDED");
    }
    if candidate_count > candidate_budget {
        push_unique(violations, "CANDIDATE_BUDGET_EXCEEDED");
    }
    (total, low, wall_time, human)
}

fn evaluate_run(
    run: &Value,
    objectives: &[Objective],
    gate_ids: &[String],
    reference_point: &[f64],
    high_fidelity_budget: u64,
    low_fidelity_budget: u64,
    wall_time_ms_budget: u64,
    human_minutes_budget: u64,
    candidate_budget: u64,
    violations: &mut Vec<&'static str>,
) -> RunReport {
    let seed = run.get("seed").and_then(Value::as_u64);
    let (high_fidelity_calls_used, low_fidelity_calls, wall_time_ms_used, human_minutes_used) =
        validate_run_accounting(
            run,
            high_fidelity_budget,
            low_fidelity_budget,
            wall_time_ms_budget,
            human_minutes_budget,
            candidate_budget,
            violations,
        );
    let Some(candidates) = run.get("candidates").and_then(Value::as_array) else {
        push_unique(violations, "CANDIDATE_LIST_MISSING");
        return RunReport {
            seed,
            candidate_count: 0,
            feasible_count: 0,
            false_pass_count: 0,
            false_reject_count: 0,
            pareto_ids: Vec::new(),
            excluded: Vec::new(),
            hypervolume_2d: None,
            high_fidelity_calls_used,
            low_fidelity_calls,
            wall_time_ms_used,
            human_minutes_used,
        };
    };

    let mut ids = HashSet::new();
    let mut evaluations = Vec::new();
    let mut false_pass_count = 0;
    let mut false_reject_count = 0;
    for candidate in candidates {
        let evaluation = evaluate_candidate(candidate, objectives, gate_ids);
        if !evaluation.declaration_valid {
            push_unique(violations, "CANDIDATE_FEASIBILITY_DECLARATION_MISSING");
        }
        if !ids.insert(evaluation.id.clone()) {
            push_unique(violations, "CANDIDATE_ID_DUPLICATED_WITHIN_RUN");
        }
        if evaluation.declared_feasible && !evaluation.feasible {
            false_pass_count += 1;
            push_unique(violations, "FALSE_FEASIBLE_DECLARATION");
        }
        if !evaluation.declared_feasible && evaluation.feasible {
            false_reject_count += 1;
        }
        evaluations.push(evaluation);
    }

    let feasible: Vec<&CandidateEvaluation> = evaluations
        .iter()
        .filter(|candidate| candidate.feasible)
        .collect();
    let vectors: Vec<Vec<f64>> = feasible
        .iter()
        .filter_map(|candidate| candidate.normalized.clone())
        .collect();
    let front_indices = pareto_indices(&vectors);
    let pareto_ids: Vec<String> = front_indices
        .iter()
        .map(|index| feasible[*index].id.clone())
        .collect();
    let front_points: Vec<Vec<f64>> = front_indices
        .iter()
        .map(|index| vectors[*index].clone())
        .collect();
    let hypervolume_2d = hypervolume_for_front(&front_points, reference_point);

    let excluded = evaluations
        .iter()
        .filter(|candidate| !candidate.feasible)
        .map(|candidate| (candidate.id.clone(), candidate.exclusion_reasons.clone()))
        .collect();

    RunReport {
        seed,
        candidate_count: candidates.len(),
        feasible_count: feasible.len(),
        false_pass_count,
        false_reject_count,
        pareto_ids,
        excluded,
        hypervolume_2d,
        high_fidelity_calls_used,
        low_fidelity_calls,
        wall_time_ms_used,
        human_minutes_used,
    }
}

pub fn evaluate_bundle(bundle: &Value) -> EvaluationReport {
    let mut violations = Vec::new();

    let fixture = bundle.get("fixture");
    let fixture_id = fixture
        .and_then(|value| get_str(value, "id"))
        .unwrap_or("UNKNOWN_FIXTURE")
        .to_owned();
    let fixture_digest = fixture.and_then(|value| get_str(value, "sha256"));
    if !is_sha256(fixture_digest) {
        push_unique(&mut violations, "FIXTURE_DIGEST_INVALID");
    }
    if get_str(bundle, "schema_version") != Some("luminous.search-eval-bundle/0.1") {
        push_unique(&mut violations, "BUNDLE_SCHEMA_VERSION_INVALID");
    }
    if fixture.and_then(|value| get_str(value, "evidence_tier")) != Some("E1") {
        push_unique(&mut violations, "FIXTURE_EVIDENCE_TIER_NOT_E1");
    }
    if fixture.and_then(|value| get_str(value, "claim_ceiling")) != Some(CLAIM_CEILING) {
        push_unique(&mut violations, "FIXTURE_CLAIM_CEILING_INVALID");
    }

    let objective_schema_digest = get_str(bundle, "objective_schema_sha256");
    if !is_sha256(objective_schema_digest) {
        push_unique(&mut violations, "OBJECTIVE_SCHEMA_DIGEST_INVALID");
    }
    let objectives = objectives_from(bundle, &mut violations);
    let reference_point: Vec<f64> = fixture
        .and_then(|value| value.get("reference_point_normalized"))
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default();
    if reference_point.len() != objectives.len()
        || reference_point
            .iter()
            .any(|value| !value.is_finite() || *value <= 1.0)
    {
        push_unique(&mut violations, "HYPERVOLUME_REFERENCE_POINT_INVALID");
    }

    let gate_ids = required_gate_ids(bundle, &mut violations);
    let Some(method_items) = bundle.get("methods").and_then(Value::as_array) else {
        push_unique(&mut violations, "METHOD_LIST_MISSING");
        return EvaluationReport {
            fixture_id,
            matched_comparison: false,
            exact_reference_method: None,
            exact_reference_front_ids: Vec::new(),
            objective_count: objectives.len(),
            methods: Vec::new(),
            unsupported_metrics: vec!["IGD+", "GD+", "epsilon"],
            violations,
        };
    };

    if method_items.len() < 2 {
        push_unique(&mut violations, "AT_LEAST_TWO_METHODS_REQUIRED");
    }
    let mut method_ids = HashSet::new();
    let mut reference_method_id = fixture
        .and_then(|value| get_str(value, "reference_method_id"))
        .map(str::to_owned);
    let enumeration_complete = fixture
        .and_then(|value| get_bool(value, "enumeration_complete"))
        .unwrap_or(false);
    if enumeration_complete && reference_method_id.is_none() {
        push_unique(&mut violations, "EXACT_REFERENCE_METHOD_MISSING");
    }
    if !enumeration_complete && reference_method_id.is_some() {
        push_unique(
            &mut violations,
            "EXACT_REFERENCE_CLAIM_WITHOUT_COMPLETE_ENUMERATION",
        );
        reference_method_id = None;
    }

    let mut budget_signature: Option<(u64, u64, u64, u64, u64)> = None;
    let mut reports = Vec::new();
    let mut exact_reference_front_ids = Vec::new();
    let mut exact_reference_found = false;

    for method in method_items {
        let id = get_str(method, "id").unwrap_or("UNKNOWN_METHOD").to_owned();
        if id == "UNKNOWN_METHOD" || !method_ids.insert(id.clone()) {
            push_unique(&mut violations, "METHOD_ID_MISSING_OR_DUPLICATED");
        }
        let algorithm = get_str(method, "algorithm")
            .unwrap_or("UNKNOWN_ALGORITHM")
            .to_owned();
        if algorithm == "UNKNOWN_ALGORITHM" {
            push_unique(&mut violations, "ALGORITHM_IDENTITY_MISSING");
        }
        if get_str(method, "fixture_sha256") != fixture_digest
            || !is_sha256(get_str(method, "code_sha256"))
            || get_str(method, "objective_schema_sha256") != objective_schema_digest
        {
            push_unique(&mut violations, "METHOD_INPUT_OR_CODE_DIGEST_MISMATCH");
        }

        let candidate_budget = get_u64(method, "candidate_budget_per_run").unwrap_or_else(|| {
            push_unique(&mut violations, "CANDIDATE_BUDGET_MISSING");
            0
        });
        let high_fidelity_budget = get_u64(method, "high_fidelity_call_budget_per_run")
            .unwrap_or_else(|| {
                push_unique(&mut violations, "HIGH_FIDELITY_BUDGET_MISSING");
                0
            });
        let low_fidelity_budget = get_u64(method, "low_fidelity_call_budget_per_run")
            .unwrap_or_else(|| {
                push_unique(&mut violations, "LOW_FIDELITY_BUDGET_MISSING");
                0
            });
        let wall_time_ms_budget =
            get_u64(method, "wall_time_ms_budget_per_run").unwrap_or_else(|| {
                push_unique(&mut violations, "WALL_TIME_BUDGET_MISSING");
                0
            });
        let human_minutes_budget =
            get_u64(method, "human_minutes_budget_per_run").unwrap_or_else(|| {
                push_unique(&mut violations, "HUMAN_TIME_BUDGET_MISSING");
                0
            });
        let signature = (
            candidate_budget,
            high_fidelity_budget,
            low_fidelity_budget,
            wall_time_ms_budget,
            human_minutes_budget,
        );
        if let Some(expected) = budget_signature {
            if expected != signature {
                push_unique(&mut violations, "METHOD_BUDGETS_NOT_MATCHED");
            }
        } else {
            budget_signature = Some(signature);
        }

        let stochastic = get_bool(method, "stochastic").unwrap_or(false);
        let enumeration_complete_for_method =
            get_bool(method, "enumeration_complete").unwrap_or(false);
        if enumeration_complete_for_method && algorithm != "EXHAUSTIVE_ENUMERATION" {
            push_unique(
                &mut violations,
                "NON_EXHAUSTIVE_METHOD_CLAIMS_EXACT_ENUMERATION",
            );
        }
        if !enumeration_complete_for_method && algorithm == "EXHAUSTIVE_ENUMERATION" {
            push_unique(
                &mut violations,
                "EXHAUSTIVE_METHOD_MISSING_ENUMERATION_FLAG",
            );
        }

        let Some(runs) = method.get("runs").and_then(Value::as_array) else {
            push_unique(&mut violations, "METHOD_RUNS_MISSING");
            reports.push(MethodReport {
                id,
                algorithm,
                stochastic,
                runs: Vec::new(),
            });
            continue;
        };
        if stochastic {
            if runs.len() < 20 {
                push_unique(&mut violations, "STOCHASTIC_RUN_COUNT_BELOW_20");
            }
            let mut seeds = HashSet::new();
            for run in runs {
                match run.get("seed").and_then(Value::as_u64) {
                    Some(seed) if seeds.insert(seed) => {}
                    _ => push_unique(&mut violations, "STOCHASTIC_SEED_MISSING_OR_DUPLICATED"),
                }
            }
        } else if runs.len() != 1 || runs[0].get("seed").is_some_and(|seed| !seed.is_null()) {
            push_unique(&mut violations, "DETERMINISTIC_RUN_SHAPE_INVALID");
        }

        if enumeration_complete_for_method {
            if !enumeration_complete {
                push_unique(
                    &mut violations,
                    "METHOD_EXACT_ENUMERATION_WITHOUT_FIXTURE_ENUM_COMPLETE",
                );
            }
            if runs.len() != 1 {
                push_unique(&mut violations, "EXACT_ENUMERATION_MUST_HAVE_ONE_RUN");
            }
            let enumerated_count = runs
                .first()
                .and_then(|run| run.get("candidates"))
                .and_then(Value::as_array)
                .map_or(0, |items| items.len()) as u64;
            if enumerated_count != candidate_budget {
                push_unique(&mut violations, "EXACT_ENUMERATION_COUNT_INCOMPLETE");
            }
            if reference_method_id.as_deref() != Some(id.as_str()) {
                push_unique(&mut violations, "EXACT_REFERENCE_METHOD_ID_MISMATCH");
            } else if enumeration_complete {
                exact_reference_found = true;
            }
        }

        let mut run_reports = Vec::new();
        for run in runs {
            run_reports.push(evaluate_run(
                run,
                &objectives,
                &gate_ids,
                &reference_point,
                high_fidelity_budget,
                low_fidelity_budget,
                wall_time_ms_budget,
                human_minutes_budget,
                candidate_budget,
                &mut violations,
            ));
        }

        if reference_method_id.as_deref() == Some(id.as_str()) {
            if let Some(run) = run_reports.first() {
                exact_reference_front_ids = run.pareto_ids.clone();
            }
            // Exact enumeration is an oracle only if every enumerated candidate
            // has a current, complete, exact evaluation and known hard gates.
            let reference_run = runs.first();
            let reference_candidates = reference_run
                .and_then(|value| value.get("candidates"))
                .and_then(Value::as_array);
            let mut exact_reference_complete = reference_candidates.is_some();
            if let Some(candidates) = reference_candidates {
                for candidate in candidates {
                    if get_str(candidate, "dependency_status") != Some("current") {
                        push_unique(&mut violations, "EXACT_REFERENCE_DEPENDENCY_NOT_CURRENT");
                        exact_reference_complete = false;
                    }
                    if normalize_objectives(candidate, &objectives).is_err() {
                        push_unique(
                            &mut violations,
                            "EXACT_REFERENCE_OBJECTIVE_VALUES_INCOMPLETE",
                        );
                        exact_reference_complete = false;
                    }
                    let gate_map = candidate.get("hard_gates").and_then(Value::as_object);
                    let all_gates_known = gate_ids.iter().all(|gate_id| {
                        matches!(
                            gate_map
                                .and_then(|map| map.get(gate_id))
                                .and_then(Value::as_str),
                            Some("passed" | "failed")
                        )
                    });
                    if !all_gates_known {
                        push_unique(&mut violations, "EXACT_REFERENCE_HARD_GATE_STATE_UNKNOWN");
                        exact_reference_complete = false;
                    }
                    let verification = candidate.get("verification");
                    let warnings = verification
                        .and_then(|value| value.get("warnings"))
                        .and_then(Value::as_array);
                    let warnings_acknowledged = warnings.is_some_and(|items| {
                        items.is_empty()
                            || verification
                                .and_then(|value| get_bool(value, "warnings_acknowledged"))
                                == Some(true)
                    });
                    let full_fidelity = is_sha256(get_str(candidate, "input_sha256"))
                        && is_sha256(get_str(candidate, "candidate_revision_sha256"))
                        && gate_evidence_is_complete(candidate, &gate_ids)
                        && warnings_acknowledged
                        && is_sha256(
                            verification.and_then(|value| get_str(value, "result_sha256")),
                        )
                        && verification.and_then(|value| get_str(value, "kind"))
                            == Some("high_fidelity")
                        && verification.and_then(|value| get_bool(value, "converged"))
                            == Some(true)
                        && verification
                            .and_then(|value| get_bool(value, "required_inputs_complete"))
                            == Some(true)
                        && verification
                            .and_then(|value| get_str(value, "validation_scope"))
                            .is_some_and(|scope| !scope.is_empty());
                    if !full_fidelity {
                        push_unique(
                            &mut violations,
                            "EXACT_REFERENCE_EVALUATION_SCOPE_INCOMPLETE",
                        );
                        exact_reference_complete = false;
                    }
                }
            }
            if !exact_reference_complete {
                push_unique(
                    &mut violations,
                    "EXACT_REFERENCE_ENUMERATION_NOT_FULLY_QUALIFIED",
                );
            }
        }

        reports.push(MethodReport {
            id,
            algorithm,
            stochastic,
            runs: run_reports,
        });
    }

    if enumeration_complete && !exact_reference_found {
        push_unique(&mut violations, "EXACT_REFERENCE_METHOD_NOT_FOUND");
    }

    let matched_comparison = method_items.len() >= 2
        && budget_signature.is_some()
        && fixture_digest.is_some()
        && objective_schema_digest.is_some()
        && !violations.iter().any(|violation| {
            matches!(
                *violation,
                "METHOD_BUDGETS_NOT_MATCHED"
                    | "METHOD_INPUT_OR_CODE_DIGEST_MISMATCH"
                    | "AT_LEAST_TWO_METHODS_REQUIRED"
            )
        });

    EvaluationReport {
        fixture_id,
        matched_comparison,
        exact_reference_method: if exact_reference_found {
            reference_method_id
        } else {
            None
        },
        exact_reference_front_ids,
        objective_count: objectives.len(),
        methods: reports,
        unsupported_metrics: vec!["IGD+", "GD+", "epsilon"],
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn candidate(
        id: &str,
        first: f64,
        second: f64,
        gate_access: &str,
        gate_fire: &str,
        kind: &str,
        reported_feasible: bool,
    ) -> Value {
        json!({
            "id": id,
            "input_sha256": digest('b'),
            "candidate_revision_sha256": digest('f'),
            "reported_feasible": reported_feasible,
            "hard_gate_evidence_sha256": {
                "step_free_task_fixture": digest('a'),
                "life_safety_task_fixture": digest('c')
            },
            "dependency_status": "current",
            "verification": {
                "kind": kind,
                "result_sha256": digest('e'),
                "converged": true,
                "required_inputs_complete": true,
                "validation_scope": "synthetic_mathematical_fixture",
                "warnings": [],
                "warnings_acknowledged": true
            },
            "hard_gates": {
                "step_free_task_fixture": gate_access,
                "life_safety_task_fixture": gate_fire
            },
            "objectives": {
                "synthetic_cost": first,
                "synthetic_discomfort": second
            }
        })
    }

    fn exact_candidates() -> Vec<Value> {
        vec![
            candidate("A", 1.0, 9.0, "passed", "passed", "high_fidelity", true),
            candidate("B", 2.0, 7.0, "passed", "passed", "high_fidelity", true),
            candidate("C", 3.0, 5.0, "passed", "passed", "high_fidelity", true),
            candidate("D", 5.0, 4.0, "passed", "passed", "high_fidelity", true),
            candidate("E", 7.0, 2.0, "passed", "passed", "high_fidelity", true),
            candidate("F", 9.0, 1.0, "passed", "passed", "high_fidelity", true),
            candidate("G", 4.0, 8.0, "passed", "passed", "high_fidelity", true),
            candidate("H", 6.0, 6.0, "passed", "passed", "high_fidelity", true),
            candidate("I", 0.0, 0.0, "failed", "passed", "high_fidelity", false),
            candidate("J", 10.0, 10.0, "passed", "passed", "high_fidelity", true),
            candidate("K", 1.0, 2.0, "passed", "failed", "high_fidelity", false),
        ]
    }

    fn make_run(seed: Option<u64>, candidates: Vec<Value>, calls: Value) -> Value {
        json!({
            "seed": seed,
            "high_fidelity_calls": calls,
            "low_fidelity_calls": 4,
            "wall_time_ms_used": 1000,
            "human_minutes_used": 10,
            "candidates": candidates
        })
    }

    fn calls(training: u64, tuning: u64, validation: u64, final_calls: u64) -> Value {
        json!({
            "training": training,
            "tuning": tuning,
            "validation": validation,
            "final": final_calls
        })
    }

    fn valid_bundle() -> Value {
        let fixture_digest = digest('a');
        let objective_digest = digest('c');
        let exact = json!({
            "id": "exact",
            "algorithm": "EXHAUSTIVE_ENUMERATION",
            "version": "fixture-1",
            "code_sha256": digest('d'),
            "fixture_sha256": fixture_digest,
            "objective_schema_sha256": objective_digest,
            "stochastic": false,
            "enumeration_complete": true,
            "candidate_budget_per_run": 11,
            "high_fidelity_call_budget_per_run": 11,
            "low_fidelity_call_budget_per_run": 8,
            "wall_time_ms_budget_per_run": 5000,
            "human_minutes_budget_per_run": 20,
            "runs": [
                make_run(None, exact_candidates(), calls(0, 0, 0, 11))
            ]
        });

        let mut nsga_candidates = vec![
            candidate("N-A", 1.0, 9.0, "passed", "passed", "high_fidelity", true),
            candidate("N-C", 3.0, 5.0, "passed", "passed", "high_fidelity", true),
            candidate("N-E", 7.0, 2.0, "passed", "passed", "high_fidelity", true),
            candidate("N-G", 4.0, 8.0, "passed", "passed", "high_fidelity", true),
            candidate("N-I", 0.1, 0.1, "failed", "passed", "high_fidelity", false),
            candidate("N-J", 0.2, 0.2, "passed", "passed", "surrogate_only", false),
            candidate(
                "N-K",
                0.3,
                0.3,
                "passed",
                "unknown",
                "surrogate_only",
                false,
            ),
        ];
        // Make one candidate a deliberately conservative false reject. It is
        // actually feasible, and the auditor reports rather than hides it.
        nsga_candidates.push(candidate(
            "N-FR",
            8.5,
            1.5,
            "passed",
            "passed",
            "high_fidelity",
            false,
        ));

        let runs: Vec<Value> = (0..20)
            .map(|seed| make_run(Some(seed), nsga_candidates.clone(), calls(3, 1, 1, 6)))
            .collect();
        let nsga = json!({
            "id": "nsga2",
            "algorithm": "NSGA-II",
            "version": "test-fixture-1",
            "code_sha256": digest('e'),
            "fixture_sha256": fixture_digest,
            "objective_schema_sha256": objective_digest,
            "stochastic": true,
            "enumeration_complete": false,
            "candidate_budget_per_run": 11,
            "high_fidelity_call_budget_per_run": 11,
            "low_fidelity_call_budget_per_run": 8,
            "wall_time_ms_budget_per_run": 5000,
            "human_minutes_budget_per_run": 20,
            "runs": runs
        });

        json!({
            "schema_version": "luminous.search-eval-bundle/0.1",
            "fixture": {
                "id": "SEARCH-EVAL-MATH-001",
                "sha256": fixture_digest,
                "evidence_tier": "E1",
                "claim_ceiling": CLAIM_CEILING,
                "enumeration_complete": true,
                "reference_method_id": "exact",
                "reference_point_normalized": [1.2, 1.2]
            },
            "objective_schema_sha256": objective_digest,
            "objectives": [
                {
                    "id": "synthetic_cost",
                    "direction": "minimize",
                    "unit": "synthetic_units",
                    "normalization_lower": 0.0,
                    "normalization_upper": 10.0
                },
                {
                    "id": "synthetic_discomfort",
                    "direction": "minimize",
                    "unit": "synthetic_units",
                    "normalization_lower": 0.0,
                    "normalization_upper": 10.0
                }
            ],
            "hard_gates": [
                {"id": "step_free_task_fixture", "authority_scope": "synthetic_boolean_gate_only"},
                {"id": "life_safety_task_fixture", "authority_scope": "synthetic_boolean_gate_only"}
            ],
            "methods": [exact, nsga]
        })
    }

    fn has_violation(report: &EvaluationReport, expected: &str) -> bool {
        report.violations.contains(&expected)
    }

    #[test]
    fn exact_small_fixture_and_matched_stochastic_method_validate() {
        let report = evaluate_bundle(&valid_bundle());
        assert!(report.is_valid(), "{:?}", report.violations);
        assert_eq!(report.exact_reference_method.as_deref(), Some("exact"));
        assert_eq!(
            report.exact_reference_front_ids,
            vec!["A", "B", "C", "D", "E", "F"]
        );
        assert_eq!(report.methods.len(), 2);
        assert_eq!(report.methods[1].runs.len(), 20);
    }

    #[test]
    fn hard_gate_failure_is_excluded_even_with_better_objectives() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][8]["objectives"]["synthetic_cost"] =
            json!(0.0);
        let report = evaluate_bundle(&bundle);
        assert!(report.is_valid(), "{:?}", report.violations);
        let exact = &report.methods[0].runs[0];
        assert!(!exact.pareto_ids.iter().any(|id| id == "I"));
        assert!(
            exact
                .excluded
                .iter()
                .any(|(id, reasons)| id == "I" && reasons.contains(&"HARD_GATE_FAILED"))
        );
    }

    #[test]
    fn false_feasible_declaration_is_a_hard_violation() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][8]["reported_feasible"] = json!(true);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
        assert!(!report.is_valid());
    }

    #[test]
    fn unknown_hard_gate_never_enters_the_feasible_front() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][10]["hard_gates"]["life_safety_task_fixture"] =
            json!("unknown");
        bundle["methods"][0]["runs"][0]["candidates"][10]["reported_feasible"] = json!(true);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
        assert!(
            !report.methods[0].runs[0]
                .pareto_ids
                .contains(&"K".to_owned())
        );
    }

    #[test]
    fn surrogate_only_candidate_is_excluded_from_final_front() {
        let report = evaluate_bundle(&valid_bundle());
        let run = &report.methods[1].runs[0];
        assert!(!run.pareto_ids.contains(&"N-J".to_owned()));
        assert!(
            run.excluded.iter().any(|(id, reasons)| id == "N-J"
                && reasons.contains(&"SURROGATE_ONLY_NOT_FINAL_EVIDENCE"))
        );
    }

    #[test]
    fn missing_feasibility_declaration_is_not_silently_treated_as_reject() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]
            .as_object_mut()
            .unwrap()
            .remove("reported_feasible");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(
            &report,
            "CANDIDATE_FEASIBILITY_DECLARATION_MISSING"
        ));
    }

    #[test]
    fn undeclared_hard_gate_result_is_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["hard_gates"]["unregistered"] =
            json!("passed");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
        assert!(
            report.methods[0].runs[0]
                .excluded
                .iter()
                .any(|(id, reasons)| {
                    id == "A" && reasons.contains(&"UNDECLARED_HARD_GATE_RESULT")
                })
        );
    }

    #[test]
    fn missing_gate_evidence_digest_cannot_support_a_pass() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["hard_gate_evidence_sha256"]
            .as_object_mut()
            .unwrap()
            .remove("life_safety_task_fixture");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn missing_candidate_revision_digest_is_not_admitted() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]
            .as_object_mut()
            .unwrap()
            .remove("candidate_revision_sha256");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn missing_solver_warning_list_is_unknown_not_empty() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["verification"]
            .as_object_mut()
            .unwrap()
            .remove("warnings");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn nonconverged_candidate_cannot_be_declared_feasible() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["verification"]["converged"] =
            json!(false);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn missing_validation_scope_cannot_be_declared_feasible() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["verification"]["validation_scope"] =
            json!("");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn stale_dependencies_are_excluded_and_false_pass_detected() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][0]["dependency_status"] = json!("stale");
        bundle["methods"][0]["runs"][0]["candidates"][0]["reported_feasible"] = json!(true);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FALSE_FEASIBLE_DECLARATION"));
    }

    #[test]
    fn methods_with_different_fixture_digests_are_not_matched() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["fixture_sha256"] = json!(digest('f'));
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(
            &report,
            "METHOD_INPUT_OR_CODE_DIGEST_MISMATCH"
        ));
        assert!(!report.matched_comparison);
    }

    #[test]
    fn unequal_high_fidelity_budgets_are_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["high_fidelity_call_budget_per_run"] = json!(10);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "METHOD_BUDGETS_NOT_MATCHED"));
    }

    #[test]
    fn unequal_low_fidelity_budgets_are_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["low_fidelity_call_budget_per_run"] = json!(7);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "METHOD_BUDGETS_NOT_MATCHED"));
    }

    #[test]
    fn low_fidelity_calls_cannot_exceed_the_predeclared_budget() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["runs"][0]["low_fidelity_calls"] = json!(9);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "LOW_FIDELITY_BUDGET_EXCEEDED"));
    }

    #[test]
    fn unequal_wall_time_budgets_are_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["wall_time_ms_budget_per_run"] = json!(4000);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "METHOD_BUDGETS_NOT_MATCHED"));
    }

    #[test]
    fn wall_time_must_be_reported_and_stay_within_budget() {
        let mut missing = valid_bundle();
        missing["methods"][1]["runs"][0]
            .as_object_mut()
            .unwrap()
            .remove("wall_time_ms_used");
        let missing_report = evaluate_bundle(&missing);
        assert!(has_violation(
            &missing_report,
            "WALL_TIME_ACCOUNTING_MISSING"
        ));

        let mut over = valid_bundle();
        over["methods"][1]["runs"][0]["wall_time_ms_used"] = json!(5001);
        let over_report = evaluate_bundle(&over);
        assert!(has_violation(&over_report, "WALL_TIME_BUDGET_EXCEEDED"));
    }

    #[test]
    fn final_solver_call_count_cannot_underreport_high_fidelity_candidates() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["runs"][0]["high_fidelity_calls"]["final"] = json!(5);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "FINAL_CALLS_UNDERREPORTED"));
    }

    #[test]
    fn hidden_surrogate_training_calls_cannot_exceed_budget() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["runs"][0]["high_fidelity_calls"]["training"] = json!(9);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "HIGH_FIDELITY_BUDGET_EXCEEDED"));
    }

    #[test]
    fn stochastic_method_needs_twenty_unique_seeds() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["runs"].as_array_mut().unwrap().pop();
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "STOCHASTIC_RUN_COUNT_BELOW_20"));
    }

    #[test]
    fn duplicate_random_seeds_are_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][1]["runs"][1]["seed"] = json!(0);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(
            &report,
            "STOCHASTIC_SEED_MISSING_OR_DUPLICATED"
        ));
    }

    #[test]
    fn exact_front_is_not_claimed_without_complete_enumeration() {
        let mut bundle = valid_bundle();
        bundle["fixture"]["enumeration_complete"] = json!(false);
        bundle["fixture"]
            .as_object_mut()
            .unwrap()
            .remove("reference_method_id");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(
            &report,
            "METHOD_EXACT_ENUMERATION_WITHOUT_FIXTURE_ENUM_COMPLETE"
        ));
        assert!(report.exact_reference_method.is_none());
    }

    #[test]
    fn exact_enumeration_count_must_match_candidate_budget() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"]
            .as_array_mut()
            .unwrap()
            .pop();
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "EXACT_ENUMERATION_COUNT_INCOMPLETE"));
    }

    #[test]
    fn malformed_reference_point_is_rejected() {
        let mut bundle = valid_bundle();
        bundle["fixture"]["reference_point_normalized"] = json!([0.8, 1.2]);
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(
            &report,
            "HYPERVOLUME_REFERENCE_POINT_INVALID"
        ));
    }

    #[test]
    fn duplicate_candidate_ids_are_rejected() {
        let mut bundle = valid_bundle();
        bundle["methods"][0]["runs"][0]["candidates"][1]["id"] = json!("A");
        let report = evaluate_bundle(&bundle);
        assert!(has_violation(&report, "CANDIDATE_ID_DUPLICATED_WITHIN_RUN"));
    }

    #[test]
    fn normalization_respects_maximize_direction() {
        let objective = Objective {
            id: "quality".to_owned(),
            direction: "maximize".to_owned(),
            lower: 0.0,
            upper: 100.0,
        };
        let candidate = json!({"objectives":{"quality":75.0}});
        assert_eq!(
            normalize_objectives(&candidate, &[objective]).unwrap(),
            vec![0.25]
        );
    }

    #[test]
    fn two_dimensional_hypervolume_is_exact_for_known_union() {
        let area = hypervolume_2d(&[[0.2, 0.8], [0.8, 0.2]], [1.2, 1.2]).unwrap();
        assert!((area - 0.64).abs() < 1e-12);
    }

    #[test]
    fn hypervolume_does_not_count_points_worse_than_reference() {
        let area = hypervolume_2d(&[[1.3, 0.2], [0.2, 1.4]], [1.2, 1.2]).unwrap();
        assert_eq!(area, 0.0);
    }

    #[test]
    fn same_objective_point_does_not_dominate_its_duplicate() {
        assert!(!dominates(&[0.2, 0.4], &[0.2, 0.4]));
    }

    #[test]
    fn output_explicitly_disclaims_building_superiority() {
        let report = evaluate_bundle(&valid_bundle());
        let output = report.to_value();
        assert_eq!(
            output["claim_limit"],
            "no_building_performance_or_sota_superiority_claim"
        );
        assert_eq!(
            output["result"],
            "benchmark_bundle_valid_mathematical_fixture_only"
        );
    }
}
