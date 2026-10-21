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
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_bottleneck_scenarios_v0_1.json";
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
    has_label(case.get("oracle").unwrap_or(&Value::Null), "causes", expected)
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
        Err(format!("JSON Schema validation failed:\n{}", errors.join("\n")))
    }
}

fn validate_cross_fields(fixture: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if as_string(fixture, "schema_version") != Some(EXPECTED_SCHEMA_VERSION) {
        errors.push(format!(
            "schema_version must be {EXPECTED_SCHEMA_VERSION}"
        ));
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
        if observations.get("reported_backlog").and_then(Value::as_bool) != Some(true) {
            errors.push(format!("{id}: reported_backlog must be explicitly true in v0.1"));
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
            errors.push(format!("{id}: intervention labels must use UPPER_SNAKE_CASE"));
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
            errors.push(format!("{id}: unknown funding requires EVIDENCE_INSUFFICIENT"));
        }
        if matches!(funding, "absent" | "unknown")
            && !prohibited.iter().any(|label| {
                label.contains("UNFUNDED") || label.contains("FUNDED_JOB")
            })
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
            Some("inspection_pending")
                if !contains_cause(case, "AUTHORIZATION_OR_SAFETY_GATE") =>
            {
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
            errors.push(format!("{id}: dispatch mismatch requires DISPATCH_OR_GEOGRAPHY"));
        }
        if as_string(observations, "wage_and_shift_status") == Some("nonviable_synthetic")
            && !contains_cause(case, "WAGE_OR_RETENTION")
        {
            errors.push(format!("{id}: nonviable conditions require WAGE_OR_RETENTION"));
        }
        if as_string(observations, "demand_status") == Some("duplicate_and_misrouted")
            && !contains_cause(case, "DEMAND_DATA_QUALITY")
        {
            errors.push(format!("{id}: duplicate demand requires DEMAND_DATA_QUALITY"));
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
        if evidence != "complete"
            && !admissible.contains(&"COLLECT_OR_REVALIDATE_EVIDENCE")
        {
            errors.push(format!(
                "{id}: non-complete evidence must permit COLLECT_OR_REVALIDATE_EVIDENCE"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
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
        corpus_id: fixture["corpus_id"].as_str().unwrap_or_default().to_string(),
        schema_version: fixture["schema_version"].as_str().unwrap_or_default().to_string(),
        case_count: fixture["cases"].as_array().map_or(0, Vec::len),
        fixture_sha256,
        schema_sha256,
        json_schema_validation: "PASS",
        cross_field_validation: "PASS",
        backlog_only_baseline: baseline_report(&fixture),
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
}
