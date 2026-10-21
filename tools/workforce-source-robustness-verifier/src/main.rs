use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-SOURCE-ROBUSTNESS-001";
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_source_robustness_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_source_robustness_v0_1.schema.json";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Scenario {
    id: String,
    description: String,
    sources: Vec<Source>,
    baseline_decision: String,
    perturbations: Vec<Perturbation>,
    expected_invariants: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Source {
    id: String,
    family: String,
    measure: String,
    value: f64,
    unit: String,
    population: String,
    geography: String,
    period_start: String,
    period_end: String,
    denominator: String,
    access_class: String,
    age_days: u32,
    #[serde(default)]
    taxonomy_version: Option<String>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Perturbation {
    id: String,
    source_id: String,
    operation: String,
    parameter: Value,
    expected_decision: String,
    rationale: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct PerturbationResult {
    perturbation_id: String,
    operation: String,
    expected_decision: String,
    observed_decision: String,
    passed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ScenarioResult {
    scenario_id: String,
    baseline_expected: String,
    baseline_observed: String,
    baseline_passed: bool,
    perturbations: Vec<PerturbationResult>,
}

#[derive(Debug, Serialize)]
struct Receipt {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    schema_sha256: String,
    scenario_count: usize,
    perturbation_count: usize,
    pass_count: usize,
    fail_count: usize,
    scenarios: Vec<ScenarioResult>,
    limitations: Vec<&'static str>,
    claim_ceiling: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_decision(value: &str) -> bool {
    matches!(
        value,
        "confirmed_task_capacity_gap"
            | "no_confirmed_shortage"
            | "job_quality_confounding"
            | "fragile_or_indeterminate"
    )
}

fn validate_schema(schema: &Value, corpus: &Value) -> Result<(), String> {
    let validator =
        validator_for(schema).map_err(|error| format!("schema cannot be compiled: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(corpus)
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
        errors.push(format!(
            "schema_version must be {EXPECTED_SCHEMA_VERSION}, got {}",
            corpus.schema_version
        ));
    }
    if corpus.benchmark_id != EXPECTED_BENCHMARK_ID {
        errors.push(format!(
            "benchmark_id must be {EXPECTED_BENCHMARK_ID}, got {}",
            corpus.benchmark_id
        ));
    }
    if !corpus.synthetic_only {
        errors.push("this verifier accepts only synthetic source records".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES ARE SYNTHETIC")
    {
        errors.push("fixture notice must identify all values as synthetic".to_string());
    }
    if !is_decision(&corpus.claim_ceiling) && corpus.claim_ceiling.trim().is_empty() {
        errors.push("claim ceiling cannot be empty".to_string());
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in &corpus.scenarios {
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("duplicate scenario ID {}", scenario.id));
        }
        if scenario.sources.is_empty() {
            errors.push(format!("{}: source list cannot be empty", scenario.id));
        }
        let mut source_ids = BTreeSet::new();
        for source in &scenario.sources {
            if !source_ids.insert(source.id.clone()) {
                errors.push(format!(
                    "{}: duplicate source ID {}",
                    scenario.id, source.id
                ));
            }
            if source.access_class != "synthetic" {
                errors.push(format!(
                    "{} / {}: non-synthetic access class is forbidden in this verifier",
                    scenario.id, source.id
                ));
            }
            if !source.value.is_finite() || source.value < 0.0 {
                errors.push(format!(
                    "{} / {}: source value must be finite and non-negative",
                    scenario.id, source.id
                ));
            }
            if source.period_start > source.period_end {
                errors.push(format!(
                    "{} / {}: period_start must not follow period_end",
                    scenario.id, source.id
                ));
            }
            if source.unit.trim().is_empty()
                || source.population.trim().is_empty()
                || source.geography.trim().is_empty()
                || source.denominator.trim().is_empty()
                || source.limitations.is_empty()
            {
                errors.push(format!(
                    "{} / {}: units, population, geography, denominator and limitations are mandatory",
                    scenario.id, source.id
                ));
            }
        }
        if !is_decision(&scenario.baseline_decision) {
            errors.push(format!(
                "{}: unsupported baseline decision {}",
                scenario.id, scenario.baseline_decision
            ));
        }
        if scenario.expected_invariants.is_empty() {
            errors.push(format!("{}: expected_invariants cannot be empty", scenario.id));
        }

        let mut perturbation_ids = BTreeSet::new();
        for perturbation in &scenario.perturbations {
            if !perturbation_ids.insert(perturbation.id.clone()) {
                errors.push(format!(
                    "{}: duplicate perturbation ID {}",
                    scenario.id, perturbation.id
                ));
            }
            if !source_ids.contains(&perturbation.source_id) {
                errors.push(format!(
                    "{} / {}: perturbation references missing source {}",
                    scenario.id, perturbation.id, perturbation.source_id
                ));
            }
            if !is_decision(&perturbation.expected_decision) {
                errors.push(format!(
                    "{} / {}: unsupported expected decision {}",
                    scenario.id, perturbation.id, perturbation.expected_decision
                ));
            }
            if perturbation.rationale.trim().is_empty() {
                errors.push(format!(
                    "{} / {}: perturbation rationale cannot be empty",
                    scenario.id, perturbation.id
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn source_is_time_sensitive(source: &Source) -> bool {
    matches!(
        source.measure.as_str(),
        "online_ads"
            | "funded_vacancies"
            | "task_ready_capacity"
            | "registered_supply"
            | "employer_hiring_intention"
            | "vacancy_duration_days"
            | "median_wage_change_pct"
            | "offer_decline_share_pct"
            | "turnover_rate_pct"
    )
}

fn derive_decision(sources: &[Source]) -> String {
    if sources.is_empty() {
        return "fragile_or_indeterminate".to_string();
    }

    // Stale decision-critical source evidence may not substantiate a current
    // workforce conclusion. The age threshold is a synthetic contract rule.
    if sources
        .iter()
        .any(|source| source_is_time_sensitive(source) && source.age_days > 180)
    {
        return "fragile_or_indeterminate".to_string();
    }

    // A taxonomy-mapping count is mapping-sensitive when alternatives disagree
    // in version or value for this same frozen synthetic population.
    let mapping: Vec<&Source> = sources
        .iter()
        .filter(|source| source.measure == "taxonomy_mapping_count")
        .collect();
    if mapping.len() > 1
        && mapping.iter().skip(1).any(|source| {
            source.value != mapping[0].value
                || source.taxonomy_version != mapping[0].taxonomy_version
                || source.population != mapping[0].population
                || source.denominator != mapping[0].denominator
        })
    {
        return "fragile_or_indeterminate".to_string();
    }

    // When decision-critical measures are compared, geography and reference
    // period must line up. This is intentionally conservative.
    let core: Vec<&Source> = sources
        .iter()
        .filter(|source| {
            matches!(
                source.measure.as_str(),
                "online_ads" | "funded_vacancies" | "task_ready_capacity" | "registered_supply"
            )
        })
        .collect();
    if core.len() > 1
        && core.iter().skip(1).any(|source| {
            source.geography != core[0].geography
                || source.period_start != core[0].period_start
                || source.period_end != core[0].period_end
        })
    {
        return "fragile_or_indeterminate".to_string();
    }

    let ads_present = sources
        .iter()
        .any(|source| source.measure == "online_ads" && source.value > 0.0);
    let offer_declines = sources
        .iter()
        .find(|source| source.measure == "offer_decline_share_pct")
        .map(|source| source.value);
    let turnover = sources
        .iter()
        .find(|source| source.measure == "turnover_rate_pct")
        .map(|source| source.value);

    if ads_present
        && matches!((offer_declines, turnover), (Some(declines), Some(turnover)) if declines >= 50.0 && turnover >= 50.0)
    {
        return "job_quality_confounding".to_string();
    }
    if ads_present
        && (offer_declines.is_some_and(|value| value >= 50.0)
            || turnover.is_some_and(|value| value >= 50.0))
    {
        return "fragile_or_indeterminate".to_string();
    }

    let funded: Vec<&Source> = sources
        .iter()
        .filter(|source| source.measure == "funded_vacancies")
        .collect();
    let capacity: Vec<&Source> = sources
        .iter()
        .filter(|source| source.measure == "task_ready_capacity")
        .collect();

    if funded.len() > 1 || capacity.len() > 1 {
        return "fragile_or_indeterminate".to_string();
    }
    if let (Some(funded), Some(capacity)) = (funded.first(), capacity.first()) {
        if funded.geography != capacity.geography
            || funded.period_start != capacity.period_start
            || funded.period_end != capacity.period_end
        {
            return "fragile_or_indeterminate".to_string();
        }
        if funded.value > capacity.value {
            return "confirmed_task_capacity_gap".to_string();
        }
        return "no_confirmed_shortage".to_string();
    }

    // A current zero on the explicitly scoped funded-position record means
    // shortage is not established in this case. It does not prove no shortage.
    if funded.len() == 1 && funded[0].value == 0.0 {
        return "no_confirmed_shortage".to_string();
    }

    // Ads or employer intentions without aligned funding/capacity evidence do
    // not warrant a confirmed shortage claim.
    if sources.iter().any(|source| {
        matches!(
            source.measure.as_str(),
            "online_ads" | "employer_hiring_intention"
        )
    }) {
        return "no_confirmed_shortage".to_string();
    }

    "fragile_or_indeterminate".to_string()
}

fn numeric_parameter(perturbation: &Perturbation) -> Result<f64, String> {
    perturbation
        .parameter
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            format!(
                "{}: operation {} requires a numeric parameter",
                perturbation.id, perturbation.operation
            )
        })
}

fn string_parameter(perturbation: &Perturbation) -> Result<String, String> {
    perturbation
        .parameter
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "{}: operation {} requires a string parameter",
                perturbation.id, perturbation.operation
            )
        })
}

fn apply_perturbation(
    scenario: &Scenario,
    perturbation: &Perturbation,
) -> Result<Vec<Source>, String> {
    let target = scenario
        .sources
        .iter()
        .find(|source| source.id == perturbation.source_id)
        .ok_or_else(|| {
            format!(
                "{}: missing source {}",
                perturbation.id, perturbation.source_id
            )
        })?;
    let mut sources = scenario.sources.clone();

    match perturbation.operation.as_str() {
        "multiply_online_ads" => {
            if target.measure != "online_ads" {
                return Err(format!(
                    "{}: multiply_online_ads requires an online_ads source",
                    perturbation.id
                ));
            }
            let factor = numeric_parameter(perturbation)?;
            if factor < 0.0 {
                return Err(format!("{}: factor must be non-negative", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.value *= factor;
            if !source.value.is_finite() {
                return Err(format!("{}: transformed count overflowed", perturbation.id));
            }
        }
        "drop_source" => {
            sources.retain(|source| source.id != perturbation.source_id);
        }
        "reduce_coverage" => {
            if target.measure != "online_ads" {
                return Err(format!(
                    "{}: reduce_coverage requires an online_ads source",
                    perturbation.id
                ));
            }
            let factor = numeric_parameter(perturbation)?;
            if !(0.0..=1.0).contains(&factor) {
                return Err(format!(
                    "{}: coverage factor must be within [0, 1]",
                    perturbation.id
                ));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.value *= factor;
        }
        "replace_employer_sample" => {
            if target.family != "employer_survey" {
                return Err(format!(
                    "{}: replace_employer_sample requires an employer_survey source",
                    perturbation.id
                ));
            }
            if string_parameter(perturbation)? != "replace_with_empty_sample" {
                return Err(format!(
                    "{}: unsupported sample replacement parameter",
                    perturbation.id
                ));
            }
            sources.retain(|source| source.id != perturbation.source_id);
        }
        "mark_stale" => {
            let age = numeric_parameter(perturbation)?;
            if age < 0.0 || age.fract() != 0.0 || age > 36500.0 {
                return Err(format!("{}: age_days must be an integer from 0 to 36500", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.age_days = age as u32;
        }
        "alternate_taxonomy_mapping" => {
            let crosswalk = string_parameter(perturbation)?;
            if crosswalk.trim().is_empty() {
                return Err(format!("{}: crosswalk identifier is empty", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.taxonomy_version = Some(crosswalk);
        }
        "introduce_job_quality_confounder" => {
            let object = perturbation
                .parameter
                .as_object()
                .ok_or_else(|| format!("{}: expected object parameter", perturbation.id))?;
            let measure = object
                .get("measure")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{}: missing measure", perturbation.id))?;
            let value = object
                .get("value")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .ok_or_else(|| format!("{}: invalid value", perturbation.id))?;
            let unit = object
                .get("unit")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{}: missing unit", perturbation.id))?;
            if !matches!(
                measure,
                "offer_decline_share_pct" | "turnover_rate_pct"
            ) {
                return Err(format!("{}: unsupported job-quality measure", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.measure = measure.to_string();
            source.value = value;
            source.unit = unit.to_string();
            source.family = "derived_synthetic".to_string();
        }
        "leave_source_family_out" => {
            let family = target.family.clone();
            sources.retain(|source| source.family != family);
        }
        "change_reference_period" => {
            let period = string_parameter(perturbation)?;
            let (start, end) = period.split_once('/').ok_or_else(|| {
                format!("{}: period parameter must be start/end", perturbation.id)
            })?;
            if start > end || start.len() != 10 || end.len() != 10 {
                return Err(format!("{}: invalid reference period", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.period_start = start.to_string();
            source.period_end = end.to_string();
        }
        "change_denominator" => {
            let denominator = string_parameter(perturbation)?;
            if denominator.trim().is_empty() {
                return Err(format!("{}: denominator is empty", perturbation.id));
            }
            let source = sources
                .iter_mut()
                .find(|source| source.id == perturbation.source_id)
                .expect("target exists in cloned sources");
            source.denominator = denominator;
        }
        other => return Err(format!("{}: unsupported operation {other}", perturbation.id)),
    }

    Ok(sources)
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    for scenario in &mut corpus.scenarios {
        scenario.sources.sort_by(|a, b| a.id.cmp(&b.id));
        scenario.perturbations.sort_by(|a, b| a.id.cmp(&b.id));
        scenario.expected_invariants.sort();
    }
}

fn evaluate(corpus: &Corpus) -> Result<(Vec<ScenarioResult>, usize, usize), String> {
    let mut results = Vec::new();
    let mut pass_count = 0usize;
    let mut fail_count = 0usize;

    for scenario in &corpus.scenarios {
        let baseline_observed = derive_decision(&scenario.sources);
        let baseline_passed = baseline_observed == scenario.baseline_decision;
        if baseline_passed {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        let mut perturbation_results = Vec::new();
        for perturbation in &scenario.perturbations {
            let transformed = apply_perturbation(scenario, perturbation)?;
            let observed_decision = derive_decision(&transformed);
            let passed = observed_decision == perturbation.expected_decision;
            if passed {
                pass_count += 1;
            } else {
                fail_count += 1;
            }
            perturbation_results.push(PerturbationResult {
                perturbation_id: perturbation.id.clone(),
                operation: perturbation.operation.clone(),
                expected_decision: perturbation.expected_decision.clone(),
                observed_decision,
                passed,
            });
        }
        results.push(ScenarioResult {
            scenario_id: scenario.id.clone(),
            baseline_expected: scenario.baseline_decision.clone(),
            baseline_observed,
            baseline_passed,
            perturbations: perturbation_results,
        });
    }
    Ok((results, pass_count, fail_count))
}

fn verify_values(schema_value: &Value, fixture_value: &Value) -> Result<Receipt, String> {
    validate_schema(schema_value, fixture_value)?;
    let mut corpus: Corpus = serde_json::from_value(fixture_value.clone())
        .map_err(|error| format!("fixture type decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);
    let (scenarios, pass_count, fail_count) = evaluate(&corpus)?;
    let perturbation_count = scenarios
        .iter()
        .map(|scenario| scenario.perturbations.len())
        .sum::<usize>();

    Ok(Receipt {
        status: if fail_count == 0 {
            "SYNTHETIC_SOURCE_ROBUSTNESS_PASS"
        } else {
            "SYNTHETIC_SOURCE_ROBUSTNESS_FAIL"
        },
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: String::new(),
        schema_sha256: String::new(),
        scenario_count: scenarios.len(),
        perturbation_count,
        pass_count,
        fail_count,
        scenarios,
        limitations: vec![
            "All values and source-generating processes are synthetic contract inputs.",
            "Decision thresholds are explicit synthetic fixture rules, not empirically calibrated South African labour-market thresholds.",
            "A passing receipt validates internal perturbation contracts only; it does not validate a real source's representativeness.",
            "The verifier does not infer sampling weights, probabilities, causal impact, real vacancy counts or a production shortage index.",
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
    let schema_value: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;
    let mut receipt = verify_values(&schema_value, &fixture_value)?;
    receipt.fixture_sha256 = sha256(&fixture_bytes);
    receipt.schema_sha256 = sha256(&schema_bytes);
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt)
            .map_err(|error| format!("cannot serialize receipt: {error}"))?
    );
    if receipt.fail_count > 0 {
        return Err(format!(
            "{} synthetic baseline/perturbation checks failed",
            receipt.fail_count
        ));
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

    const FIXTURE: &str = include_str!(
        "../../../docs/research/fixtures/workforce_source_robustness_v0_1.json"
    );
    const SCHEMA: &str = include_str!(
        "../../../docs/research/schemas/workforce_source_robustness_v0_1.schema.json"
    );

    fn fixture_and_schema() -> (Value, Value) {
        (
            serde_json::from_str(FIXTURE).expect("fixture JSON parses"),
            serde_json::from_str(SCHEMA).expect("schema JSON parses"),
        )
    }

    #[test]
    fn schema_and_all_synthetic_perturbations_pass() {
        let (fixture, schema) = fixture_and_schema();
        let receipt = verify_values(&schema, &fixture).expect("synthetic contract verifies");
        assert_eq!(receipt.status, "SYNTHETIC_SOURCE_ROBUSTNESS_PASS");
        assert_eq!(receipt.scenario_count, 6);
        assert_eq!(receipt.perturbation_count, 11);
        assert_eq!(receipt.fail_count, 0);
        assert_eq!(receipt.pass_count, 17);
    }

    #[test]
    fn multiplying_online_ads_does_not_change_decision_class_by_itself() {
        let (fixture, schema) = fixture_and_schema();
        let receipt = verify_values(&schema, &fixture).expect("fixture verifies");
        for scenario in receipt.scenarios {
            for result in scenario
                .perturbations
                .iter()
                .filter(|result| result.operation == "multiply_online_ads")
            {
                assert_eq!(result.observed_decision, scenario.baseline_observed);
                assert!(result.passed);
            }
        }
    }

    #[test]
    fn removing_funded_evidence_never_preserves_a_confirmed_gap() {
        let (fixture_value, schema) = fixture_and_schema();
        let corpus: Corpus =
            serde_json::from_value(fixture_value.clone()).expect("fixture typed");
        let scenario = corpus
            .scenarios
            .iter()
            .find(|scenario| scenario.id == "SR-001")
            .expect("SR-001 exists");
        let mut perturbed = scenario.clone();
        perturbed.sources.retain(|source| source.id != "FUNDED");
        assert_ne!(
            derive_decision(&perturbed.sources),
            "confirmed_task_capacity_gap"
        );
        let _ = schema;
    }

    #[test]
    fn taxonomy_and_period_mismatch_are_fragile() {
        let (fixture, schema) = fixture_and_schema();
        let receipt = verify_values(&schema, &fixture).expect("fixture verifies");
        for id in ["SR-004", "SR-005", "SR-006"] {
            let scenario = receipt
                .scenarios
                .iter()
                .find(|scenario| scenario.scenario_id == id)
                .expect("scenario exists");
            assert_eq!(scenario.baseline_observed, "fragile_or_indeterminate");
        }
    }

    #[test]
    fn unknown_schema_fields_and_observed_claim_mutations_fail() {
        let (mut fixture, schema) = fixture_and_schema();
        fixture["extra_property"] = json!(true);
        assert!(verify_values(&schema, &fixture).is_err());

        let (mut fixture, schema) = fixture_and_schema();
        fixture["synthetic_only"] = Value::Bool(false);
        assert!(verify_values(&schema, &fixture).is_err());
    }

    #[test]
    fn duplicate_source_ids_fail_cross_field_validation() {
        let (mut fixture, schema) = fixture_and_schema();
        let first_source = fixture["scenarios"][0]["sources"][0].clone();
        fixture["scenarios"][0]["sources"]
            .as_array_mut()
            .expect("sources array")
            .push(first_source);
        assert!(verify_values(&schema, &fixture).is_err());
    }
}
