use jsonschema::validator_for;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_CORPUS_ID: &str = "SA-WORKFORCE-DIAG-001-V0.1";
const EXPECTED_CASE_COUNT: usize = 12;
const FIXTURE_DEFAULT: &str = "docs/research/fixtures/workforce_bottleneck_scenarios_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_bottleneck_scenarios_v0_1.schema.json";

#[derive(Debug, Serialize, PartialEq)]
struct BaselineReport {
    name: &'static str,
    rule: &'static str,
    backlog_cases: usize,
    cases_with_skill_capacity_in_oracle: usize,
    false_shortage_cases: usize,
    false_shortage_rate_among_positive_predictions: f64,
    exact_cause_set_matches: usize,
    false_shortage_case_ids: Vec<String>,
    limitations: Vec<&'static str>,
}

#[derive(Debug, Serialize, PartialEq)]
struct RuleBaselineReport {
    name: &'static str,
    cases_scored: usize,
    exact_cause_set_matches: usize,
    mismatch_case_ids: Vec<String>,
    limitations: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    corpus_id: String,
    schema_version: String,
    case_count: usize,
    fixture_sha256: String,
    schema_sha256: String,
    json_schema_validation: &'static str,
    cross_field_validation: &'static str,
    backlog_only_baseline: BaselineReport,
    transparent_rule_baseline: RuleBaselineReport,
    claim_ceiling: &'static str,
}

fn as_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn has_label(value: &Value, key: &str, expected: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(|labels| labels.iter().any(|label| label.as_str() == Some(expected)))
}

fn contains_cause(case: &Value, expected: &str) -> bool {
    has_label(
        case.get("oracle").unwrap_or(&Value::Null),
        "causes",
        expected,
    )
}

fn valid_intervention_label(label: &str) -> bool {
    !label.is_empty()
        && label.as_bytes()[0].is_ascii_uppercase()
        && label
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_schema(schema: &Value, fixture: &Value) -> Result<(), String> {
    let validator =
        validator_for(schema).map_err(|error| format!("schema compilation failed: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(fixture)
        .map(|error| format!("{}: {error}", error.instance_path()))
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

fn validate_cross_fields(fixture: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if as_string(fixture, "schema_version") != Some(EXPECTED_SCHEMA_VERSION) {
        errors.push(format!("schema_version must be {EXPECTED_SCHEMA_VERSION}"));
    }
    if as_string(fixture, "corpus_id") != Some(EXPECTED_CORPUS_ID) {
        errors.push(format!("corpus_id must be {EXPECTED_CORPUS_ID}"));
    }
    if !as_string(fixture, "claim_ceiling")
        .is_some_and(|claim| claim.contains("SYNTHETIC_TEST_ONLY"))
    {
        errors.push("claim_ceiling must include SYNTHETIC_TEST_ONLY".to_string());
    }

    let cases = fixture.get("cases").and_then(Value::as_array);
    let Some(cases) = cases else {
        return Err(vec!["cases must be an array".to_string()]);
    };
    if cases.len() != EXPECTED_CASE_COUNT {
        errors.push(format!(
            "expected exactly {EXPECTED_CASE_COUNT} cases, got {}",
            cases.len()
        ));
    }

    let mut ids = BTreeSet::new();
    for case in cases {
        let id = as_string(case, "id").unwrap_or("<missing-id>");
        if id.len() != 7 || !id.starts_with("WBD-") {
            errors.push(format!("{id}: invalid WBD-NNN case ID"));
        }
        if !ids.insert(id.to_string()) {
            errors.push(format!("{id}: duplicate case ID"));
        }
        if case.get("synthetic_only").and_then(Value::as_bool) != Some(true) {
            errors.push(format!("{id}: synthetic_only must be true"));
        }

        let observations = case.get("observations").unwrap_or(&Value::Null);
        if observations
            .get("reported_backlog")
            .and_then(Value::as_bool)
            != Some(true)
        {
            errors.push(format!(
                "{id}: reported_backlog must be explicitly true in v0.1"
            ));
        }
        let oracle = case.get("oracle").unwrap_or(&Value::Null);
        let causes = oracle
            .get("causes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if causes.is_empty() {
            errors.push(format!("{id}: oracle cause set cannot be empty"));
        }
        let cause_strings: Vec<&str> = causes.iter().filter_map(Value::as_str).collect();
        if cause_strings.iter().collect::<BTreeSet<_>>().len() != cause_strings.len() {
            errors.push(format!("{id}: duplicate cause label"));
        }

        let admissible: Vec<&str> = oracle
            .get("admissible_interventions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let prohibited: Vec<&str> = oracle
            .get("prohibited_interventions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        if admissible.is_empty() {
            errors.push(format!("{id}: admissible intervention set cannot be empty"));
        }
        if prohibited.is_empty() {
            errors.push(format!("{id}: prohibited intervention set cannot be empty"));
        }
        if admissible.iter().any(|label| prohibited.contains(label)) {
            errors.push(format!(
                "{id}: intervention cannot be both admissible and prohibited"
            ));
        }
        if admissible
            .iter()
            .chain(prohibited.iter())
            .any(|label| !valid_intervention_label(label))
        {
            errors.push(format!(
                "{id}: intervention labels must use UPPER_SNAKE_CASE"
            ));
        }

        let required = observations
            .get("required_task_capacity")
            .and_then(Value::as_u64);
        let available = observations
            .get("available_task_ready_capacity")
            .and_then(Value::as_u64);
        if matches!((required, available), (Some(r), Some(a)) if r > a)
            && !contains_cause(case, "SKILLS_CAPACITY")
        {
            errors.push(format!(
                "{id}: explicit task-ready capacity shortfall requires SKILLS_CAPACITY"
            ));
        }

        let funding = as_string(observations, "funding_status").unwrap_or("");
        if funding == "absent" && !contains_cause(case, "FUNDING_OR_PAYER") {
            errors.push(format!("{id}: absent funding requires FUNDING_OR_PAYER"));
        }
        if funding == "unknown" && !contains_cause(case, "EVIDENCE_INSUFFICIENT") {
            errors.push(format!(
                "{id}: unknown funding requires EVIDENCE_INSUFFICIENT"
            ));
        }
        if matches!(funding, "absent" | "unknown")
            && !prohibited
                .iter()
                .any(|label| label.contains("UNFUNDED") || label.contains("FUNDED_JOB"))
        {
            errors.push(format!(
                "{id}: absent/unknown funding must prohibit claiming the work is funded"
            ));
        }

        if as_string(observations, "tools_and_spares") == Some("critical_spare_missing")
            && !contains_cause(case, "TOOLS_OR_SPARES")
        {
            errors.push(format!("{id}: missing spare requires TOOLS_OR_SPARES"));
        }
        match as_string(observations, "authorization_status") {
            Some("inspection_pending") if !contains_cause(case, "AUTHORIZATION_OR_SAFETY_GATE") => {
                errors.push(format!(
                    "{id}: pending inspection requires AUTHORIZATION_OR_SAFETY_GATE"
                ));
            }
            Some("unknown") if !contains_cause(case, "EVIDENCE_INSUFFICIENT") => {
                errors.push(format!(
                    "{id}: unknown authorization requires EVIDENCE_INSUFFICIENT"
                ));
            }
            _ => {}
        }
        if as_string(observations, "dispatch_status") == Some("travel_shift_mismatch")
            && !contains_cause(case, "DISPATCH_OR_GEOGRAPHY")
        {
            errors.push(format!(
                "{id}: dispatch mismatch requires DISPATCH_OR_GEOGRAPHY"
            ));
        }
        if as_string(observations, "wage_and_shift_status") == Some("nonviable_synthetic")
            && !contains_cause(case, "WAGE_OR_RETENTION")
        {
            errors.push(format!(
                "{id}: nonviable conditions require WAGE_OR_RETENTION"
            ));
        }
        if as_string(observations, "demand_status") == Some("duplicate_and_misrouted")
            && !contains_cause(case, "DEMAND_DATA_QUALITY")
        {
            errors.push(format!(
                "{id}: duplicate demand requires DEMAND_DATA_QUALITY"
            ));
        }
        if as_string(observations, "credential_status") == Some("present_but_scope_mismatch")
            && !contains_cause(case, "PRACTICAL_COMPETENCE_UNVERIFIED")
        {
            errors.push(format!(
                "{id}: credential scope mismatch requires PRACTICAL_COMPETENCE_UNVERIFIED"
            ));
        }
        if as_string(observations, "practical_assessment_status") == Some("missing")
            && !contains_cause(case, "PRACTICAL_COMPETENCE_UNVERIFIED")
        {
            errors.push(format!(
                "{id}: missing practical assessment requires PRACTICAL_COMPETENCE_UNVERIFIED"
            ));
        }
        let evidence = as_string(observations, "evidence_status").unwrap_or("");
        if matches!(evidence, "stale_and_conflicting" | "incomplete")
            && !contains_cause(case, "EVIDENCE_INSUFFICIENT")
        {
            errors.push(format!(
                "{id}: stale/conflicting or incomplete evidence requires EVIDENCE_INSUFFICIENT"
            ));
        }
        if evidence != "complete" && !admissible.contains(&"COLLECT_OR_REVALIDATE_EVIDENCE") {
            errors.push(format!(
                "{id}: non-complete evidence must permit COLLECT_OR_REVALIDATE_EVIDENCE"
            ));
        }
        errors.extend(validate_action_labels(case));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn skill_capacity_gap(case: &Value) -> bool {
    let observations = &case["observations"];
    let required = observations
        .get("required_task_capacity")
        .and_then(Value::as_u64);
    let available = observations
        .get("available_task_ready_capacity")
        .and_then(Value::as_u64);
    matches!((required, available), (Some(r), Some(a)) if r > a)
}

fn action_is_admissible(case: &Value, action: &str) -> Option<bool> {
    let observations = &case["observations"];
    let funding = as_string(observations, "funding_status").unwrap_or("");
    let evidence = as_string(observations, "evidence_status").unwrap_or("");
    let authorization = as_string(observations, "authorization_status").unwrap_or("");
    let supervisor_available = observations
        .get("qualified_supervisor_available")
        .and_then(Value::as_bool)
        == Some(true);
    let has_skill_gap = skill_capacity_gap(case);
    let funded = funding == "confirmed_synthetic";
    let scope_is_safe = matches!(authorization, "valid_for_scope" | "supervised_scope_valid");

    match action {
        "SUPERVISED_SKILL_DEVELOPMENT" => Some(
            has_skill_gap
                && funded
                && supervisor_available
                && scope_is_safe
                && evidence == "complete",
        ),
        "QUALIFIED_REDEPLOYMENT_OR_FUNDED_HIRING" => {
            Some(has_skill_gap && funded && evidence == "complete")
        }
        "HUMAN_AI_DIAGNOSTIC_ASSIST" => Some(
            observations.get("novice_present").and_then(Value::as_bool) == Some(true)
                && supervisor_available
                && authorization == "supervised_scope_valid"
                && funded
                && evidence == "complete"
                && as_string(observations, "tools_and_spares") == Some("available"),
        ),
        "ESTABLISH_LAWFUL_FUNDING_AND_PROCUREMENT" => Some(funding == "absent"),
        "PROCURE_OR_QUALIFY_VALID_PART_SUBSTITUTE" => {
            Some(as_string(observations, "tools_and_spares") == Some("critical_spare_missing"))
        }
        "OBTAIN_QUALIFIED_INSPECTION_OR_AUTHORIZATION" | "BLOCK_EXECUTION_UNTIL_GATE_PASSES" => {
            Some(authorization == "inspection_pending")
        }
        "REDESIGN_ROUTE_OR_SHIFT_WITH_WORKER_INPUT" => {
            Some(as_string(observations, "dispatch_status") == Some("travel_shift_mismatch"))
        }
        "REVIEW_COMPENSATION_SHIFT_SAFETY_AND_RETENTION_WITH_WORKERS" => {
            Some(as_string(observations, "wage_and_shift_status") == Some("nonviable_synthetic"))
        }
        "COLLECT_OR_REVALIDATE_EVIDENCE" => Some(evidence != "complete"),
        "DEDUPE_AND_VALIDATE_DEMAND" => {
            Some(as_string(observations, "demand_status") == Some("duplicate_and_misrouted"))
        }
        "SCOPED_PRACTICAL_ASSESSMENT" => Some(
            as_string(observations, "credential_status") == Some("present_but_scope_mismatch")
                || as_string(observations, "practical_assessment_status") == Some("missing"),
        ),
        "COMBINED_INTERVENTIONS" => Some(
            has_skill_gap
                && as_string(observations, "tools_and_spares") == Some("critical_spare_missing"),
        ),
        _ => None,
    }
}

fn action_is_prohibited(case: &Value, action: &str) -> Option<bool> {
    let observations = &case["observations"];
    let funding = as_string(observations, "funding_status").unwrap_or("");
    let evidence = as_string(observations, "evidence_status").unwrap_or("");
    let authorization = as_string(observations, "authorization_status").unwrap_or("");
    let causes = rule_based_causes(case);
    let multiple_causes = causes.len() > 1;
    let has_skill_gap = causes.contains("SKILLS_CAPACITY");
    let funded = funding == "confirmed_synthetic";
    let duplicate_demand =
        as_string(observations, "demand_status") == Some("duplicate_and_misrouted");
    let interacting = causes.contains("INTERACTING_BOTTLENECKS");
    let a_mandatory_gate_is_blocked = !funded
        || authorization == "inspection_pending"
        || as_string(observations, "tools_and_spares") == Some("critical_spare_missing")
        || skill_capacity_gap(case);

    match action {
        "CLAIM_SHORTAGE_PROVEN_BY_VACANCY_COUNT_ALONE"
        | "ASSERT_REGIONAL_SKILLS_SHORTAGE_FROM_LOCAL_DELAY_ALONE"
        | "MARK_REPAIR_COMPLETE_WITHOUT_VERIFICATION"
        | "BYPASS_INSPECTION"
        | "BYPASS_AUTHORIZATION"
        | "TREAT_AI_ADVICE_AS_AUTHORIZATION"
        | "INVENT_PROBABILITIES"
        | "UNSUPERVISED_REGULATED_EXECUTION"
        | "COUNT_AI_OUTPUT_AS_COMPETENCE"
        | "USE_UNPAID_LABOUR_AS_FIX"
        | "TREAT_CREDENTIAL_AS_PROOF_OF_ALL_TASK_COMPETENCE" => Some(true),
        "COUNT_AS_FUNDED_JOB" | "SCHEDULE_UNFUNDED_WORK_AS_CONFIRMED" => {
            Some(matches!(funding, "absent" | "unknown"))
        }
        "RECOMMEND_TRAINING_AS_PRIMARY_REMEDY" => Some(!has_skill_gap || !funded),
        "LABEL_AS_SKILLS_SHORTAGE_WITHOUT_EVIDENCE" => {
            Some(!has_skill_gap || evidence != "complete")
        }
        "ASSERT_SINGLE_ROOT_CAUSE" => Some(multiple_causes || evidence != "complete"),
        "CLAIM_SKILLS_ARE_THE_ONLY_CAUSE" => Some(multiple_causes || evidence != "complete"),
        "CLAIM_ONE_INTERVENTION_UNLOCKS_COMPLETION"
        | "MARK_WORK_COMPLETE_BEFORE_BOTH_GATES_PASS" => Some(interacting),
        "MARK_WORK_COMPLETE_BEFORE_ALL_GATES_PASS" => Some(a_mandatory_gate_is_blocked),
        "STAFF_TO_RAW_TICKET_COUNT" | "REPORT_DUPLICATES_AS_DISTINCT_REPAIRS" => {
            Some(duplicate_demand)
        }
        "ASSUME_AUTHORIZATION" => Some(!matches!(
            authorization,
            "valid_for_scope" | "supervised_scope_valid"
        )),
        _ => None,
    }
}

fn validate_action_labels(case: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let id = as_string(case, "id").unwrap_or("<missing-id>");
    let oracle = case.get("oracle").unwrap_or(&Value::Null);
    for action in oracle
        .get("admissible_interventions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        match action_is_admissible(case, action) {
            Some(true) => {}
            Some(false) => errors.push(format!(
                "{id}: admissible intervention {action} fails its declared preconditions"
            )),
            None => errors.push(format!(
                "{id}: no fail-closed precondition rule for admissible intervention {action}"
            )),
        }
    }
    for action in oracle
        .get("prohibited_interventions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        match action_is_prohibited(case, action) {
            Some(true) => {}
            Some(false) => errors.push(format!(
                "{id}: prohibited intervention {action} is not justified by its guard condition"
            )),
            None => errors.push(format!(
                "{id}: no fail-closed prohibition rule for intervention {action}"
            )),
        }
    }
    errors
}

fn baseline_report(fixture: &Value) -> BaselineReport {
    let cases = fixture["cases"].as_array().expect("validated cases array");
    let backlog_cases: Vec<&Value> = cases
        .iter()
        .filter(|case| case["observations"]["reported_backlog"] == Value::Bool(true))
        .collect();
    let false_shortage_case_ids: Vec<String> = backlog_cases
        .iter()
        .filter(|case| !contains_cause(case, "SKILLS_CAPACITY"))
        .map(|case| case["id"].as_str().unwrap_or("<missing-id>").to_string())
        .collect();
    let cases_with_skill_capacity_in_oracle = cases
        .iter()
        .filter(|case| contains_cause(case, "SKILLS_CAPACITY"))
        .count();
    let exact_cause_set_matches = cases
        .iter()
        .filter(|case| {
            case["observations"]["reported_backlog"] == Value::Bool(true)
                && case["oracle"]["causes"].as_array().is_some_and(|causes| {
                    causes.len() == 1
                        && causes.first().and_then(Value::as_str) == Some("SKILLS_CAPACITY")
                })
        })
        .count();
    let false_shortage_rate = if backlog_cases.is_empty() {
        0.0
    } else {
        false_shortage_case_ids.len() as f64 / backlog_cases.len() as f64
    };

    BaselineReport {
        name: "B1_BACKLOG_ONLY",
        rule: "reported_backlog=true => predict SKILLS_CAPACITY as the only cause",
        backlog_cases: backlog_cases.len(),
        cases_with_skill_capacity_in_oracle,
        false_shortage_cases: false_shortage_case_ids.len(),
        false_shortage_rate_among_positive_predictions: false_shortage_rate,
        exact_cause_set_matches,
        false_shortage_case_ids,
        limitations: vec![
            "Deliberately weak synthetic negative-control baseline; not an industry product.",
            "This is not a candidate model result.",
            "The fixture is not representative workforce data and has no causal or employment validity.",
        ],
    }
}

fn rule_based_causes(case: &Value) -> BTreeSet<String> {
    let observations = &case["observations"];
    let mut causes = BTreeSet::new();

    match as_string(observations, "funding_status") {
        Some("absent") => {
            causes.insert("FUNDING_OR_PAYER".to_string());
        }
        Some("unknown") => {
            // Unknown is an epistemic state, not evidence that a payer is absent.
            causes.insert("EVIDENCE_INSUFFICIENT".to_string());
        }
        _ => {}
    }

    let required = observations
        .get("required_task_capacity")
        .and_then(Value::as_u64);
    let available = observations
        .get("available_task_ready_capacity")
        .and_then(Value::as_u64);
    if matches!((required, available), (Some(r), Some(a)) if r > a) {
        causes.insert("SKILLS_CAPACITY".to_string());
    }
    if as_string(observations, "tools_and_spares") == Some("critical_spare_missing") {
        causes.insert("TOOLS_OR_SPARES".to_string());
    }
    match as_string(observations, "authorization_status") {
        Some("inspection_pending") => {
            causes.insert("AUTHORIZATION_OR_SAFETY_GATE".to_string());
        }
        Some("unknown") => {
            causes.insert("EVIDENCE_INSUFFICIENT".to_string());
        }
        _ => {}
    }
    if as_string(observations, "dispatch_status") == Some("travel_shift_mismatch") {
        causes.insert("DISPATCH_OR_GEOGRAPHY".to_string());
    }
    if as_string(observations, "wage_and_shift_status") == Some("nonviable_synthetic") {
        causes.insert("WAGE_OR_RETENTION".to_string());
    }
    if as_string(observations, "demand_status") == Some("duplicate_and_misrouted") {
        causes.insert("DEMAND_DATA_QUALITY".to_string());
    }
    if as_string(observations, "credential_status") == Some("present_but_scope_mismatch")
        || as_string(observations, "practical_assessment_status") == Some("missing")
    {
        causes.insert("PRACTICAL_COMPETENCE_UNVERIFIED".to_string());
    }
    if matches!(
        as_string(observations, "evidence_status"),
        Some("stale_and_conflicting" | "incomplete")
    ) {
        causes.insert("EVIDENCE_INSUFFICIENT".to_string());
    }

    let confirmed_blocker_count = [
        "SKILLS_CAPACITY",
        "FUNDING_OR_PAYER",
        "TOOLS_OR_SPARES",
        "AUTHORIZATION_OR_SAFETY_GATE",
        "DISPATCH_OR_GEOGRAPHY",
        "WAGE_OR_RETENTION",
        "DEMAND_DATA_QUALITY",
        "PRACTICAL_COMPETENCE_UNVERIFIED",
    ]
    .iter()
    .filter(|cause| causes.contains(***cause))
    .count();
    if confirmed_blocker_count >= 2 {
        causes.insert("INTERACTING_BOTTLENECKS".to_string());
    }
    causes
}

fn rule_baseline_report(fixture: &Value) -> RuleBaselineReport {
    let cases = fixture["cases"].as_array().expect("validated cases array");
    let mismatch_case_ids: Vec<String> = cases
        .iter()
        .filter(|case| {
            let expected: BTreeSet<String> = case["oracle"]["causes"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            rule_based_causes(case) != expected
        })
        .map(|case| case["id"].as_str().unwrap_or("<missing-id>").to_string())
        .collect();
    RuleBaselineReport {
        name: "B3_TRANSPARENT_RULES",
        cases_scored: cases.len(),
        exact_cause_set_matches: cases.len() - mismatch_case_ids.len(),
        mismatch_case_ids,
        limitations: vec![
            "Rules are authored against this handcrafted fixture; same-corpus agreement is a contract self-check, not held-out accuracy.",
            "It does not select or authorize interventions and is not a field-tested diagnosis system.",
        ],
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let fixture_path = args.next().unwrap_or_else(|| FIXTURE_DEFAULT.to_string());
    let schema_path = args.next().unwrap_or_else(|| SCHEMA_DEFAULT.to_string());

    let fixture_bytes =
        fs::read(&fixture_path).map_err(|error| format!("cannot read {fixture_path}: {error}"))?;
    let schema_bytes =
        fs::read(&schema_path).map_err(|error| format!("cannot read {schema_path}: {error}"))?;
    let fixture_sha256 = format!("{:x}", Sha256::digest(&fixture_bytes));
    let schema_sha256 = format!("{:x}", Sha256::digest(&schema_bytes));
    let fixture: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("fixture JSON is invalid: {error}"))?;
    let schema: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;

    validate_schema(&schema, &fixture)?;
    validate_cross_fields(&fixture).map_err(|errors| errors.join("\n"))?;

    let report = Report {
        status: "STRUCTURAL_AND_CROSS_FIELD_PASS_NOT_FIELD_VALIDATION",
        corpus_id: fixture["corpus_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        schema_version: fixture["schema_version"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        case_count: fixture["cases"].as_array().map_or(0, Vec::len),
        fixture_sha256,
        schema_sha256,
        json_schema_validation: "PASS",
        cross_field_validation: "PASS",
        backlog_only_baseline: baseline_report(&fixture),
        transparent_rule_baseline: rule_baseline_report(&fixture),
        claim_ceiling: "SYNTHETIC SOFTWARE-CONTRACT EVIDENCE ONLY; no real workforce or service claim",
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot serialize receipt: {error}"))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("workforce-bottleneck-verifier: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../docs/research/fixtures/workforce_bottleneck_scenarios_v0_1.json"
        ))
        .expect("fixture JSON parses")
    }

    fn schema() -> Value {
        serde_json::from_str(include_str!(
            "../../../docs/research/schemas/workforce_bottleneck_scenarios_v0_1.schema.json"
        ))
        .expect("schema JSON parses")
    }

    fn validate_for_test(value: &Value) -> Result<(), String> {
        validate_schema(&schema(), value)?;
        validate_cross_fields(value).map_err(|errors| errors.join("\n"))
    }

    #[test]
    fn frozen_fixture_passes_json_schema_and_cross_field_checks() {
        validate_for_test(&fixture()).expect("frozen fixture should validate");
    }

    #[test]
    fn backlog_only_baseline_exposes_false_shortage_diagnoses() {
        let value = fixture();
        validate_for_test(&value).expect("fixture must validate");
        let report = baseline_report(&value);
        assert_eq!(report.backlog_cases, 12);
        assert_eq!(report.cases_with_skill_capacity_in_oracle, 4);
        assert_eq!(report.false_shortage_cases, 8);
        assert_eq!(
            report.false_shortage_case_ids,
            vec![
                "WBD-002".to_string(),
                "WBD-003".to_string(),
                "WBD-004".to_string(),
                "WBD-005".to_string(),
                "WBD-006".to_string(),
                "WBD-007".to_string(),
                "WBD-010".to_string(),
                "WBD-011".to_string(),
            ]
        );
        assert_eq!(report.exact_cause_set_matches, 2);
    }

    #[test]
    fn transparent_rules_match_the_handcrafted_oracle_contract() {
        let value = fixture();
        validate_for_test(&value).expect("fixture must validate");
        let report = rule_baseline_report(&value);
        assert_eq!(report.cases_scored, 12);
        assert_eq!(report.exact_cause_set_matches, 12);
        assert!(report.mismatch_case_ids.is_empty());
    }

    #[test]
    fn json_schema_rejects_unknown_top_level_fields() {
        let mut value = fixture();
        value["unrecognized_field"] = Value::Bool(true);
        assert!(validate_schema(&schema(), &value).is_err());
    }

    #[test]
    fn cross_field_contract_rejects_duplicate_case_ids() {
        let mut value = fixture();
        let first_id = value["cases"][0]["id"].clone();
        value["cases"][1]["id"] = first_id;
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn cross_field_contract_rejects_non_synthetic_case() {
        let mut value = fixture();
        value["cases"][0]["synthetic_only"] = Value::Bool(false);
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn cross_field_contract_rejects_capacity_shortfall_without_skill_cause() {
        let mut value = fixture();
        value["cases"][0]["oracle"]["causes"] = json!([]);
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn strict_schema_rejects_new_observation_field_without_schema_change() {
        let mut value = fixture();
        value["cases"][0]["observations"]["unrecognized_observation"] = Value::Bool(true);
        assert!(validate_schema(&schema(), &value).is_err());
    }

    #[test]
    fn intervention_guard_rejects_training_as_a_fix_for_unfunded_non_shortage_work() {
        let mut value = fixture();
        value["cases"][1]["oracle"]["admissible_interventions"]
            .as_array_mut()
            .expect("admissible interventions array")
            .push(Value::String("SUPERVISED_SKILL_DEVELOPMENT".to_string()));
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn intervention_guard_rejects_unrecognized_actions_fail_closed() {
        let mut value = fixture();
        value["cases"][0]["oracle"]["admissible_interventions"]
            .as_array_mut()
            .expect("admissible interventions array")
            .push(Value::String("MAGIC_AUTOMATIC_JOB_CREATION".to_string()));
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn intervention_guard_rejects_funded_job_claim_when_funding_is_confirmed() {
        let mut value = fixture();
        value["cases"][0]["oracle"]["prohibited_interventions"]
            .as_array_mut()
            .expect("prohibited interventions array")
            .push(Value::String("COUNT_AS_FUNDED_JOB".to_string()));
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn capacity_signal_mutation_changes_diagnosis() {
        let mut value = fixture();
        let original = rule_based_causes(&value["cases"][0]);
        assert!(original.contains("SKILLS_CAPACITY"));

        value["cases"][0]["observations"]["required_task_capacity"] = json!(2);
        value["cases"][0]["observations"]["available_task_ready_capacity"] = json!(4);
        let mutated = rule_based_causes(&value["cases"][0]);
        assert!(!mutated.contains("SKILLS_CAPACITY"));
        assert_ne!(original, mutated);
    }

    #[test]
    fn supervised_training_fails_closed_without_qualified_supervisor() {
        let mut value = fixture();
        value["cases"][0]["observations"]["qualified_supervisor_available"] = Value::Bool(false);
        assert!(validate_cross_fields(&value).is_err());
    }

    #[test]
    fn unknown_authorization_is_uncertainty_not_confirmed_gate_failure() {
        let mut value = fixture();
        value["cases"][3]["observations"]["authorization_status"] = json!("unknown");
        let inferred = rule_based_causes(&value["cases"][3]);
        assert!(inferred.contains("EVIDENCE_INSUFFICIENT"));
        assert!(!inferred.contains("AUTHORIZATION_OR_SAFETY_GATE"));
    }

    #[test]
    fn unknown_funding_is_uncertainty_not_confirmed_absence() {
        let mut value = fixture();
        value["cases"][1]["observations"]["funding_status"] = json!("unknown");
        let inferred = rule_based_causes(&value["cases"][1]);
        assert!(inferred.contains("EVIDENCE_INSUFFICIENT"));
        assert!(!inferred.contains("FUNDING_OR_PAYER"));
        assert_eq!(
            action_is_admissible(
                &value["cases"][1],
                "ESTABLISH_LAWFUL_FUNDING_AND_PROCUREMENT"
            ),
            Some(false)
        );
        assert_eq!(
            action_is_admissible(&value["cases"][1], "COLLECT_OR_REVALIDATE_EVIDENCE"),
            Some(true)
        );
    }

    #[test]
    fn diagnostic_invariants_hold_across_cartesian_blocker_sweep() {
        let funding_states = ["confirmed_synthetic", "absent", "unknown"];
        let tool_states = ["available", "critical_spare_missing", "unknown"];
        let authorization_states = [
            "valid_for_scope",
            "inspection_pending",
            "supervised_scope_valid",
            "unknown",
        ];
        let dispatch_states = ["feasible", "travel_shift_mismatch", "unknown"];
        let wage_states = ["viable", "nonviable_synthetic", "unknown"];
        let demand_states = ["validated", "duplicate_and_misrouted", "conflicting"];
        let evidence_states = ["complete", "partial", "stale_and_conflicting", "incomplete"];
        let template = fixture()["cases"][0].clone();
        let mut checked = 0usize;

        for skill_gap in [false, true] {
            for funding in funding_states {
                for tools in tool_states {
                    for authorization in authorization_states {
                        for dispatch in dispatch_states {
                            for wage in wage_states {
                                for demand in demand_states {
                                    for evidence in evidence_states {
                                        let mut case = template.clone();
                                        case["observations"]["required_task_capacity"] =
                                            json!(if skill_gap { 3 } else { 1 });
                                        case["observations"]["available_task_ready_capacity"] =
                                            json!(if skill_gap { 1 } else { 3 });
                                        case["observations"]["funding_status"] = json!(funding);
                                        case["observations"]["tools_and_spares"] = json!(tools);
                                        case["observations"]["authorization_status"] =
                                            json!(authorization);
                                        case["observations"]["dispatch_status"] = json!(dispatch);
                                        case["observations"]["wage_and_shift_status"] = json!(wage);
                                        case["observations"]["demand_status"] = json!(demand);
                                        case["observations"]["evidence_status"] = json!(evidence);

                                        let inferred = rule_based_causes(&case);
                                        assert_eq!(
                                            inferred.contains("SKILLS_CAPACITY"),
                                            skill_gap,
                                            "skill gap invariant failed: skill_gap={:?}, funding={:?}, tools={:?}, authorization={:?}, dispatch={:?}, wage={:?}, demand={:?}, evidence={:?}",
                                            skill_gap,
                                            funding,
                                            tools,
                                            authorization,
                                            dispatch,
                                            wage,
                                            demand,
                                            evidence
                                        );
                                        assert_eq!(
                                            inferred.contains("FUNDING_OR_PAYER"),
                                            funding == "absent",
                                            "funding absence must be distinguished from unknown: funding={}",
                                            funding
                                        );
                                        if funding == "unknown" {
                                            assert!(inferred.contains("EVIDENCE_INSUFFICIENT"));
                                        }
                                        assert_eq!(
                                            inferred.contains("TOOLS_OR_SPARES"),
                                            tools == "critical_spare_missing"
                                        );
                                        assert_eq!(
                                            inferred.contains("AUTHORIZATION_OR_SAFETY_GATE"),
                                            authorization == "inspection_pending"
                                        );
                                        if authorization == "unknown" {
                                            assert!(inferred.contains("EVIDENCE_INSUFFICIENT"));
                                        }
                                        assert_eq!(
                                            inferred.contains("DISPATCH_OR_GEOGRAPHY"),
                                            dispatch == "travel_shift_mismatch"
                                        );
                                        assert_eq!(
                                            inferred.contains("WAGE_OR_RETENTION"),
                                            wage == "nonviable_synthetic"
                                        );
                                        assert_eq!(
                                            inferred.contains("DEMAND_DATA_QUALITY"),
                                            demand == "duplicate_and_misrouted"
                                        );
                                        if matches!(
                                            evidence,
                                            "stale_and_conflicting" | "incomplete"
                                        ) {
                                            assert!(inferred.contains("EVIDENCE_INSUFFICIENT"));
                                        }
                                        let confirmed_blocker_count = [
                                            skill_gap,
                                            funding == "absent",
                                            tools == "critical_spare_missing",
                                            authorization == "inspection_pending",
                                            dispatch == "travel_shift_mismatch",
                                            wage == "nonviable_synthetic",
                                            demand == "duplicate_and_misrouted",
                                        ]
                                        .into_iter()
                                        .filter(|is_blocked| *is_blocked)
                                        .count();
                                        let interaction_expected = confirmed_blocker_count >= 2;
                                        assert_eq!(
                                            inferred.contains("INTERACTING_BOTTLENECKS"),
                                            interaction_expected
                                        );
                                        checked += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(checked, 7_776);
    }
}
