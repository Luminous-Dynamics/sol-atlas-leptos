use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-RECOVERY-001";
const MAX_INTERVENTIONS: usize = 16;
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_recovery_scenarios_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_recovery_v0_1.schema.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Resources {
    qualified_person_shifts: u64,
    supervisor_hours: u64,
    tool_kits: u64,
    dispatch_hours: u64,
    cash_zar_cents: u64,
}

impl Resources {
    fn checked_add(&self, other: &Self, scenario_id: &str) -> Result<Self, String> {
        macro_rules! sum {
            ($field:ident) => {
                self.$field.checked_add(other.$field).ok_or_else(|| {
                    format!(
                        "{scenario_id}: integer overflow while adding {}",
                        stringify!($field)
                    )
                })?
            };
        }
        Ok(Self {
            qualified_person_shifts: sum!(qualified_person_shifts),
            supervisor_hours: sum!(supervisor_hours),
            tool_kits: sum!(tool_kits),
            dispatch_hours: sum!(dispatch_hours),
            cash_zar_cents: sum!(cash_zar_cents),
        })
    }

    fn fits(&self, caps: &Self) -> bool {
        self.qualified_person_shifts <= caps.qualified_person_shifts
            && self.supervisor_hours <= caps.supervisor_hours
            && self.tool_kits <= caps.tool_kits
            && self.dispatch_hours <= caps.dispatch_hours
            && self.cash_zar_cents <= caps.cash_zar_cents
    }

    fn componentwise_le(&self, other: &Self) -> bool {
        self.qualified_person_shifts <= other.qualified_person_shifts
            && self.supervisor_hours <= other.supervisor_hours
            && self.tool_kits <= other.tool_kits
            && self.dispatch_hours <= other.dispatch_hours
            && self.cash_zar_cents <= other.cash_zar_cents
    }

    fn strictly_better(&self, other: &Self) -> bool {
        self.componentwise_le(other) && self != other
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Costs {
    staff_minutes: u64,
    delay_hours: u64,
    worker_burden_hours: u64,
}

impl Costs {
    fn checked_add(&self, other: &Self, scenario_id: &str) -> Result<Self, String> {
        macro_rules! sum {
            ($field:ident) => {
                self.$field.checked_add(other.$field).ok_or_else(|| {
                    format!(
                        "{scenario_id}: integer overflow while adding {}",
                        stringify!($field)
                    )
                })?
            };
        }
        Ok(Self {
            staff_minutes: sum!(staff_minutes),
            delay_hours: sum!(delay_hours),
            worker_burden_hours: sum!(worker_burden_hours),
        })
    }

    fn componentwise_le(&self, other: &Self) -> bool {
        self.staff_minutes <= other.staff_minutes
            && self.delay_hours <= other.delay_hours
            && self.worker_burden_hours <= other.worker_burden_hours
    }

    fn strictly_better(&self, other: &Self) -> bool {
        self.componentwise_le(other) && self != other
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Services {
    essential_water_units: u64,
    essential_power_units: u64,
    health_units: u64,
    housing_units: u64,
    other_service_units: u64,
}

impl Services {
    fn checked_add_service(
        &mut self,
        category: &str,
        units: u64,
        scenario_id: &str,
    ) -> Result<(), String> {
        let target = match category {
            "essential_water_units" => &mut self.essential_water_units,
            "essential_power_units" => &mut self.essential_power_units,
            "health_units" => &mut self.health_units,
            "housing_units" => &mut self.housing_units,
            "other_service_units" => &mut self.other_service_units,
            _ => return Err(format!("{scenario_id}: unsupported service category {category}")),
        };
        *target = target.checked_add(units).ok_or_else(|| {
            format!("{scenario_id}: integer overflow while adding {category}")
        })?;
        Ok(())
    }

    fn meets(&self, floors: &Self) -> bool {
        self.essential_water_units >= floors.essential_water_units
            && self.essential_power_units >= floors.essential_power_units
            && self.health_units >= floors.health_units
            && self.housing_units >= floors.housing_units
            && self.other_service_units >= floors.other_service_units
    }

    fn componentwise_ge(&self, other: &Self) -> bool {
        self.essential_water_units >= other.essential_water_units
            && self.essential_power_units >= other.essential_power_units
            && self.health_units >= other.health_units
            && self.housing_units >= other.housing_units
            && self.other_service_units >= other.other_service_units
    }

    fn strictly_better(&self, other: &Self) -> bool {
        self.componentwise_ge(other) && self != other
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Intervention {
    id: String,
    case_id: String,
    description: String,
    available: bool,
    eligibility: String,
    prerequisite_state: String,
    funding_state: String,
    authorization_state: String,
    work_quality_state: String,
    service_category: String,
    service_units: u64,
    resource_demand: Resources,
    cost_demand: Costs,
    requires_selected_interventions: Vec<String>,
}

impl Intervention {
    fn selectable(&self) -> bool {
        self.available
            && self.eligibility == "permitted"
            && self.prerequisite_state == "satisfied"
            && self.funding_state == "confirmed"
            && self.authorization_state == "authorized"
            && self.work_quality_state == "viable"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct FrontierItem {
    selected_intervention_ids: Vec<String>,
    service_benefits: Services,
    resource_use: Resources,
    cost_vector: Costs,
    added_bundle_ids: Vec<String>,
    removed_bundle_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Scenario {
    id: String,
    description: String,
    baseline_plan_ids: Vec<String>,
    resource_caps: Resources,
    worker_burden_cap_hours: Option<u64>,
    service_floors: Services,
    interventions: Vec<Intervention>,
    expected_baseline_status: String,
    expected_baseline_codes: Vec<String>,
    expected_recovery_status: String,
    expected_infeasibility_codes: Vec<String>,
    expected_feasible_portfolio_count: usize,
    expected_frontier: Vec<FrontierItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ScenarioReceipt {
    scenario_id: String,
    baseline_status: String,
    baseline_failure_codes: Vec<String>,
    recovery_status: String,
    recovery_failure_codes: Vec<String>,
    subset_count: usize,
    feasible_portfolio_count: usize,
    frontier: Vec<FrontierItem>,
    expected_frontier: Vec<FrontierItem>,
    passed: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    schema_sha256: String,
    scenario_count: usize,
    scenarios_passed: usize,
    scenarios_failed: usize,
    scenarios: Vec<ScenarioReceipt>,
    limitations: Vec<&'static str>,
    claim_ceiling: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
        errors.push("v0.1 recovery corpus must remain synthetic-only".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES BELOW ARE SYNTHETIC DISRUPTION CONTRACT TEST DATA")
    {
        errors.push("fixture notice must identify synthetic disruption data".to_string());
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in &corpus.scenarios {
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("duplicate scenario ID {}", scenario.id));
        }
        if scenario.interventions.is_empty() || scenario.interventions.len() > MAX_INTERVENTIONS {
            errors.push(format!(
                "{}: candidate count must be 1..={MAX_INTERVENTIONS}",
                scenario.id
            ));
        }
        let mut intervention_ids = BTreeSet::new();
        for item in &scenario.interventions {
            if !intervention_ids.insert(item.id.clone()) {
                errors.push(format!("{}: duplicate intervention ID {}", scenario.id, item.id));
            }
            if item.requires_selected_interventions.contains(&item.id) {
                errors.push(format!("{}: self dependency on {}", scenario.id, item.id));
            }
        }
        for item in &scenario.interventions {
            for dependency in &item.requires_selected_interventions {
                if !intervention_ids.contains(dependency) {
                    errors.push(format!(
                        "{} / {}: unknown dependency {dependency}",
                        scenario.id, item.id
                    ));
                }
            }
        }
        for baseline_id in &scenario.baseline_plan_ids {
            if !intervention_ids.contains(baseline_id) {
                errors.push(format!(
                    "{}: baseline refers to unknown intervention {baseline_id}",
                    scenario.id
                ));
            }
        }
        if scenario.baseline_plan_ids.iter().collect::<BTreeSet<_>>().len()
            != scenario.baseline_plan_ids.len()
        {
            errors.push(format!("{}: duplicate baseline ID", scenario.id));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors.join("\n")) }
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    for scenario in &mut corpus.scenarios {
        scenario.baseline_plan_ids.sort();
        scenario.interventions.sort_by(|a, b| a.id.cmp(&b.id));
        for item in &mut scenario.interventions {
            item.requires_selected_interventions.sort();
        }
        scenario.expected_baseline_codes.sort();
        scenario.expected_infeasibility_codes.sort();
        for item in &mut scenario.expected_frontier {
            item.selected_intervention_ids.sort();
            item.added_bundle_ids.sort();
            item.removed_bundle_ids.sort();
        }
        scenario.expected_frontier.sort_by(|a, b| {
            a.selected_intervention_ids.cmp(&b.selected_intervention_ids)
        });
    }
}

fn baseline_assessment(scenario: &Scenario) -> Result<(String, Vec<String>), String> {
    let baseline_ids: BTreeSet<&str> =
        scenario.baseline_plan_ids.iter().map(String::as_str).collect();
    let mut resource_use = Resources::default();
    let mut services = Services::default();
    let mut costs = Costs::default();

    for id in &scenario.baseline_plan_ids {
        let item = scenario
            .interventions
            .iter()
            .find(|candidate| candidate.id == *id)
            .ok_or_else(|| format!("{}: missing baseline item {id}", scenario.id))?;
        // Keep resource reservations visible even if a planned bundle was disrupted.
        resource_use = resource_use.checked_add(&item.resource_demand, &scenario.id)?;
        costs = costs.checked_add(&item.cost_demand, &scenario.id)?;
        if item.selectable() {
            services.checked_add_service(&item.service_category, item.service_units, &scenario.id)?;
        }
    }

    let mut codes = BTreeSet::new();
    if !services.meets(&scenario.service_floors) {
        codes.insert("service_floor_unmet".to_string());
    }
    if !resource_use.fits(&scenario.resource_caps) {
        codes.insert("resource_capacity_infeasible".to_string());
    }
    if scenario
        .worker_burden_cap_hours
        .is_some_and(|cap| costs.worker_burden_hours > cap)
    {
        codes.insert("worker_burden_cap_infeasible".to_string());
    }
    let status = baseline_status(&codes).to_string();
    Ok((status, codes.into_iter().collect()))
}

fn baseline_status(codes: &BTreeSet<String>) -> &'static str {
    let service = codes.contains("service_floor_unmet");
    let resource = codes.contains("resource_capacity_infeasible");
    let burden = codes.contains("worker_burden_cap_infeasible");
    match (service, resource, burden) {
        (false, false, false) => "feasible",
        (true, false, false) => "service_floor_unmet",
        (false, true, false) => "resource_capacity_infeasible",
        (false, false, true) => "worker_burden_cap_infeasible",
        (true, true, false) => "service_and_resource_infeasible",
        (true, false, true) => "service_and_worker_burden_infeasible",
        (false, true, true) => "resource_and_worker_burden_infeasible",
        (true, true, true) => "multiple_constraints_infeasible",
    }
}

fn selected_structure_valid(selected: &[&Intervention]) -> bool {
    if selected.iter().any(|item| !item.selectable()) {
        return false;
    }
    let mut cases = BTreeSet::new();
    for item in selected {
        if !cases.insert(item.case_id.as_str()) {
            return false;
        }
    }
    let ids: BTreeSet<&str> = selected.iter().map(|item| item.id.as_str()).collect();
    selected.iter().all(|item| {
        item.requires_selected_interventions
            .iter()
            .all(|dep| ids.contains(dep.as_str()))
    })
}

fn component_dominates(left: &FrontierItem, right: &FrontierItem) -> bool {
    let services_ok = left.service_benefits.componentwise_ge(&right.service_benefits);
    let resources_ok = left.resource_use.componentwise_le(&right.resource_use);
    let costs_ok = left.cost_vector.componentwise_le(&right.cost_vector);
    let changes_ok = left.added_bundle_ids.len() <= right.added_bundle_ids.len()
        && left.removed_bundle_ids.len() <= right.removed_bundle_ids.len();
    let strict = left.service_benefits.strictly_better(&right.service_benefits)
        || left.resource_use.strictly_better(&right.resource_use)
        || left.cost_vector.strictly_better(&right.cost_vector)
        || left.added_bundle_ids.len() < right.added_bundle_ids.len()
        || left.removed_bundle_ids.len() < right.removed_bundle_ids.len();
    services_ok && resources_ok && costs_ok && changes_ok && strict
}

fn create_portfolio(scenario: &Scenario, selected: &[&Intervention]) -> Result<FrontierItem, String> {
    let mut resources = Resources::default();
    let mut costs = Costs::default();
    let mut services = Services::default();
    let mut selected_ids = Vec::new();

    for item in selected {
        resources = resources.checked_add(&item.resource_demand, &scenario.id)?;
        costs = costs.checked_add(&item.cost_demand, &scenario.id)?;
        services.checked_add_service(&item.service_category, item.service_units, &scenario.id)?;
        selected_ids.push(item.id.clone());
    }
    selected_ids.sort();

    let baseline: BTreeSet<&str> =
        scenario.baseline_plan_ids.iter().map(String::as_str).collect();
    let selected_set: BTreeSet<&str> = selected_ids.iter().map(String::as_str).collect();
    let mut added: Vec<String> = selected_set
        .difference(&baseline)
        .map(|id| (*id).to_string())
        .collect();
    let mut removed: Vec<String> = baseline
        .difference(&selected_set)
        .map(|id| (*id).to_string())
        .collect();
    added.sort();
    removed.sort();

    Ok(FrontierItem {
        selected_intervention_ids: selected_ids,
        service_benefits: services,
        resource_use: resources,
        cost_vector: costs,
        added_bundle_ids: added,
        removed_bundle_ids: removed,
    })
}

fn solve_recovery(
    scenario: &Scenario,
) -> Result<(String, Vec<String>, usize, Vec<FrontierItem>), String> {
    let n = scenario.interventions.len();
    if n > MAX_INTERVENTIONS {
        return Err(format!(
            "{}: oracle_limit_exceeded; {n} candidates exceeds {MAX_INTERVENTIONS}",
            scenario.id
        ));
    }
    let subset_count = 1usize
        .checked_shl(n as u32)
        .ok_or_else(|| format!("{}: oracle_limit_exceeded", scenario.id))?;

    let mut floor_achievable = false;
    let mut resource_blocked = false;
    let mut burden_blocked = false;
    let mut feasible_count = 0usize;
    let mut feasible = Vec::new();

    for mask in 0..subset_count {
        let selected: Vec<&Intervention> = scenario
            .interventions
            .iter()
            .enumerate()
            .filter(|(idx, _)| mask & (1usize << idx) != 0)
            .map(|(_, item)| item)
            .collect();

        if !selected_structure_valid(&selected) {
            continue;
        }
        let portfolio = create_portfolio(scenario, &selected)?;
        if !portfolio.service_benefits.meets(&scenario.service_floors) {
            continue;
        }
        floor_achievable = true;
        let resource_ok = portfolio.resource_use.fits(&scenario.resource_caps);
        let burden_ok = scenario
            .worker_burden_cap_hours
            .is_none_or(|cap| portfolio.cost_vector.worker_burden_hours <= cap);
        if !resource_ok {
            resource_blocked = true;
        }
        if !burden_ok {
            burden_blocked = true;
        }
        if resource_ok && burden_ok {
            feasible_count += 1;
            feasible.push(portfolio);
        }
    }

    if feasible_count > 0 {
        feasible.sort_by(|a, b| a.selected_intervention_ids.cmp(&b.selected_intervention_ids));
        let frontier = feasible
            .iter()
            .filter(|candidate| {
                !feasible.iter().any(|other| {
                    other.selected_intervention_ids != candidate.selected_intervention_ids
                        && component_dominates(other, candidate)
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut frontier = frontier;
        frontier.sort_by(|a, b| a.selected_intervention_ids.cmp(&b.selected_intervention_ids));
        return Ok(("feasible".to_string(), Vec::new(), feasible_count, frontier));
    }

    if !floor_achievable {
        return Ok((
            "infeasible_essential_floor".to_string(),
            vec!["essential_service_floor_unmet".to_string()],
            0,
            Vec::new(),
        ));
    }

    let mut codes = Vec::new();
    if resource_blocked {
        codes.push("resource_capacity_infeasible".to_string());
    }
    if burden_blocked {
        codes.push("worker_burden_cap_infeasible".to_string());
    }
    codes.sort();
    let status = match (resource_blocked, burden_blocked) {
        (true, true) => "infeasible_resource_and_worker_burden",
        (true, false) => "infeasible_resource_capacity",
        (false, true) => "infeasible_worker_burden",
        (false, false) => {
            return Err(format!(
                "{}: no feasible portfolio but no blocking constraint was identified",
                scenario.id
            ));
        }
    };
    Ok((status.to_string(), codes, 0, Vec::new()))
}

fn verify_values(
    schema: &Value,
    fixture: &Value,
    fixture_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<Report, String> {
    validate_schema(schema, fixture)?;
    let mut corpus: Corpus = serde_json::from_value(fixture.clone())
        .map_err(|error| format!("recovery fixture decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let mut receipts = Vec::new();
    for scenario in &corpus.scenarios {
        let (baseline_status, baseline_codes) = baseline_assessment(scenario)?;
        let (recovery_status, mut recovery_codes, feasible_count, frontier) =
            solve_recovery(scenario)?;
        recovery_codes.sort();
        let passed = baseline_status == scenario.expected_baseline_status
            && baseline_codes == scenario.expected_baseline_codes
            && recovery_status == scenario.expected_recovery_status
            && recovery_codes == scenario.expected_infeasibility_codes
            && feasible_count == scenario.expected_feasible_portfolio_count
            && frontier == scenario.expected_frontier;

        receipts.push(ScenarioReceipt {
            scenario_id: scenario.id.clone(),
            baseline_status,
            baseline_failure_codes: baseline_codes,
            recovery_status,
            recovery_failure_codes: recovery_codes,
            subset_count: 1usize << scenario.interventions.len(),
            feasible_portfolio_count: feasible_count,
            frontier,
            expected_frontier: scenario.expected_frontier.clone(),
            passed,
        });
    }
    receipts.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
    let scenarios_passed = receipts.iter().filter(|receipt| receipt.passed).count();
    let scenarios_failed = receipts.len() - scenarios_passed;

    Ok(Report {
        status: if scenarios_failed == 0 {
            "SYNTHETIC_RECOVERY_ORACLE_PASS"
        } else {
            "SYNTHETIC_RECOVERY_ORACLE_FAIL"
        },
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: sha256(fixture_bytes),
        schema_sha256: sha256(schema_bytes),
        scenario_count: receipts.len(),
        scenarios_passed,
        scenarios_failed,
        scenarios: receipts,
        limitations: vec![
            "All disruption conditions, intervention bundles and outcome units are synthetic.",
            "The exact recovery frontier is a tiny-instance oracle, not a production rescheduler.",
            "No individual worker-reliability prediction or invasive employee-monitoring data is used.",
            "A passing fixture does not establish real service resilience, jobs created or SOTA superiority.",
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
    if report.scenarios_failed > 0 {
        return Err(format!("{} recovery cases disagree with reference", report.scenarios_failed));
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

    const FIXTURE: &str = include_str!(
        "../../../docs/research/fixtures/workforce_recovery_scenarios_v0_1.json"
    );
    const SCHEMA: &str = include_str!(
        "../../../docs/research/schemas/workforce_recovery_v0_1.schema.json"
    );

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
            .expect("recovery contract evaluates")
    }

    fn find<'a>(report: &'a Report, id: &str) -> &'a ScenarioReceipt {
        report.scenarios.iter().find(|item| item.scenario_id == id).expect("scenario exists")
    }

    #[test]
    fn exact_recovery_oracle_matches_all_frozen_cases() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(report.status, "SYNTHETIC_RECOVERY_ORACLE_PASS");
        assert_eq!(report.scenario_count, 5);
        assert_eq!(report.scenarios_passed, 5);
        assert_eq!(report.scenarios_failed, 0);
    }

    #[test]
    fn tool_failure_is_reported_and_recovery_replaces_the_unavailable_bundle() {
        let (fixture, schema) = loaded();
        let full_report = report(&fixture, &schema);
        let receipt = find(&full_report, "RC-001");
        assert_eq!(receipt.baseline_status, "service_and_resource_infeasible");
        assert_eq!(
            receipt.frontier[0].added_bundle_ids,
            vec!["POWER_MANUAL".to_string()]
        );
        assert_eq!(
            receipt.frontier[0].removed_bundle_ids,
            vec!["POWER_STANDARD".to_string()]
        );
    }

    #[test]
    fn a_higher_service_floor_invalidates_the_frozen_plan() {
        let (fixture, schema) = loaded();
        let receipt = find(&report(&fixture, &schema), "RC-002");
        assert_eq!(receipt.baseline_status, "service_floor_unmet");
        assert_eq!(receipt.recovery_status, "feasible");
        assert_eq!(
            receipt.frontier[0].selected_intervention_ids,
            vec!["POWER_NO_TOOL".to_string(), "WATER_FAST".to_string()]
        );
    }

    #[test]
    fn resource_authority_and_worker_burden_fail_closed() {
        let (fixture, schema) = loaded();
        let receipt = report(&fixture, &schema);
        assert_eq!(find(&receipt, "RC-003").recovery_status, "infeasible_resource_capacity");
        assert_eq!(find(&receipt, "RC-004").recovery_status, "infeasible_essential_floor");
        assert_eq!(find(&receipt, "RC-005").recovery_status, "infeasible_worker_burden");
        assert!(find(&receipt, "RC-004").frontier.is_empty());
        assert!(find(&receipt, "RC-005").frontier.is_empty());
    }

    #[test]
    fn input_order_permutation_preserves_the_frontier_and_receipt() {
        let (fixture, schema) = loaded();
        let original = report(&fixture, &schema);
        let mut permuted = fixture.clone();
        permuted["scenarios"].as_array_mut().expect("scenarios").reverse();
        for scenario in permuted["scenarios"].as_array_mut().expect("scenarios") {
            scenario["interventions"].as_array_mut().expect("interventions").reverse();
            scenario["baseline_plan_ids"].as_array_mut().expect("baseline IDs").reverse();
        }
        let reordered = report(&permuted, &schema);
        assert_eq!(original.scenarios, reordered.scenarios);
    }

    #[test]
    fn adding_the_missing_supervisor_capacity_restores_feasibility() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][2]["resource_caps"]["supervisor_hours"] = json!(2);
        let report = report(&fixture, &schema);
        let recovered = find(&report, "RC-003");
        assert_eq!(recovered.recovery_status, "feasible");
        assert_eq!(recovered.feasible_portfolio_count, 1);
        assert!(recovered.frontier[0]
            .service_benefits
            .meets(&Services {
                essential_water_units: 1,
                essential_power_units: 1,
                ..Services::default()
            }));
    }

    #[test]
    fn more_resources_do_not_override_unknown_funding() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][3]["resource_caps"]["qualified_person_shifts"] = json!(u64::MAX);
        fixture["scenarios"][3]["resource_caps"]["supervisor_hours"] = json!(u64::MAX);
        fixture["scenarios"][3]["resource_caps"]["dispatch_hours"] = json!(u64::MAX);
        fixture["scenarios"][3]["resource_caps"]["cash_zar_cents"] = json!(u64::MAX);
        let report = report(&fixture, &schema);
        let recovery = find(&report, "RC-004");
        assert_eq!(recovery.recovery_status, "infeasible_essential_floor");
        assert!(recovery.frontier.is_empty());
    }

    #[test]
    fn unknown_dependency_fails_closed() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][0]["interventions"][1]["requires_selected_interventions"] =
            json!(["NO_SUCH_BUNDLE"]);
        let fixture_bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes).is_err());
    }

    #[test]
    fn integer_overflow_fails_closed() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][0]["interventions"][0]["resource_demand"]["cash_zar_cents"] =
            json!(u64::MAX);
        fixture["scenarios"][0]["interventions"][1]["resource_demand"]["cash_zar_cents"] =
            json!(u64::MAX);
        let fixture_bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(&schema).expect("schema serializes");
        assert!(verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes).is_err());
    }

    #[test]
    fn unknown_funding_cannot_be_selected_to_restore_an_essential_floor() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][0]["interventions"][0]["funding_state"] = json!("unknown");
        let report = report(&fixture, &schema);
        let recovery = find(&report, "RC-001");
        assert_eq!(recovery.recovery_status, "infeasible_essential_floor");
        assert!(recovery.frontier.is_empty());
        assert!(!recovery
            .frontier
            .iter()
            .any(|item| item.selected_intervention_ids.contains(&"WATER_LOW".to_string())));
    }
}
