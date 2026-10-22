use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-PORTFOLIO-001";
const MAX_INTERVENTIONS: usize = 20;
const MAX_SUBSETS: usize = 1 << MAX_INTERVENTIONS;
const FIXTURE_DEFAULT: &str = "docs/research/fixtures/workforce_portfolio_scenarios_v0_1.json";
const SCHEMA_DEFAULT: &str = "docs/research/schemas/workforce_portfolio_v0_1.schema.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct PortfolioVector {
    qualified_person_shifts: u64,
    supervisor_hours: u64,
    tool_kits: u64,
    dispatch_hours: u64,
    cash_zar_cents: u64,
    staff_minutes: u64,
    delay_hours: u64,
    worker_burden_hours: u64,
}

impl PortfolioVector {
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
            staff_minutes: sum!(staff_minutes),
            delay_hours: sum!(delay_hours),
            worker_burden_hours: sum!(worker_burden_hours),
        })
    }

    fn fits(&self, caps: &ResourceCaps) -> bool {
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
            && self.staff_minutes <= other.staff_minutes
            && self.delay_hours <= other.delay_hours
            && self.worker_burden_hours <= other.worker_burden_hours
    }

    fn componentwise_strictly_better(&self, other: &Self) -> bool {
        self.componentwise_le(other) && self != other
    }

    fn as_resource_caps(&self) -> ResourceCaps {
        ResourceCaps {
            qualified_person_shifts: self.qualified_person_shifts,
            supervisor_hours: self.supervisor_hours,
            tool_kits: self.tool_kits,
            dispatch_hours: self.dispatch_hours,
            cash_zar_cents: self.cash_zar_cents,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceCaps {
    qualified_person_shifts: u64,
    supervisor_hours: u64,
    tool_kits: u64,
    dispatch_hours: u64,
    cash_zar_cents: u64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct ServiceVector {
    essential_water_units: u64,
    essential_power_units: u64,
    health_units: u64,
    housing_units: u64,
    other_service_units: u64,
}

impl ServiceVector {
    fn meets(&self, floors: &Self) -> bool {
        self.essential_water_units >= floors.essential_water_units
            && self.essential_power_units >= floors.essential_power_units
            && self.health_units >= floors.health_units
            && self.housing_units >= floors.housing_units
            && self.other_service_units >= floors.other_service_units
    }

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
            other => {
                return Err(format!(
                    "{scenario_id}: unsupported service category {other}"
                ));
            }
        };
        *target = target.checked_add(units).ok_or_else(|| {
            format!("{scenario_id}: integer overflow while accumulating {category}")
        })?;
        Ok(())
    }

    fn dominates(&self, other: &Self) -> bool {
        self.essential_water_units >= other.essential_water_units
            && self.essential_power_units >= other.essential_power_units
            && self.health_units >= other.health_units
            && self.housing_units >= other.housing_units
            && self.other_service_units >= other.other_service_units
    }

    fn strictly_better_than(&self, other: &Self) -> bool {
        self.dominates(other) && self != other
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Intervention {
    id: String,
    case_id: String,
    description: String,
    eligibility: String,
    prerequisite_state: String,
    funding_state: String,
    authorization_state: String,
    work_quality_state: String,
    service_category: String,
    service_units: u64,
    resource_demand: PortfolioVector,
    requires_selected_interventions: Vec<String>,
}

impl Intervention {
    fn selectable(&self) -> bool {
        self.eligibility == "permitted"
            && self.prerequisite_state == "satisfied"
            && self.funding_state == "confirmed"
            && self.authorization_state == "authorized"
            && self.work_quality_state == "viable"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct PortfolioSummary {
    selected_intervention_ids: Vec<String>,
    service_benefits: ServiceVector,
    resource_use: ResourceCaps,
    cost_vector: PortfolioVector,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Scenario {
    id: String,
    description: String,
    resource_caps: ResourceCaps,
    worker_burden_cap_hours: Option<u64>,
    service_floors: ServiceVector,
    interventions: Vec<Intervention>,
    expected_status: String,
    expected_infeasibility_codes: Vec<String>,
    expected_feasible_portfolio_count: usize,
    expected_frontier: Vec<PortfolioSummary>,
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
    status: String,
    infeasibility_codes: Vec<String>,
    subset_count: usize,
    feasible_portfolio_count: usize,
    frontier: Vec<PortfolioSummary>,
    expected_frontier: Vec<PortfolioSummary>,
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
    if !corpus.synthetic_only || corpus.claim_ceiling != "synthetic_contract_only" {
        errors.push("v0.1 portfolio corpus must remain synthetic-only".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES BELOW ARE SYNTHETIC PORTFOLIO CONTRACT TEST DATA")
    {
        errors.push("fixture notice must explicitly identify synthetic portfolio data".to_string());
    }

    let mut scenario_ids = BTreeSet::new();
    for scenario in &corpus.scenarios {
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("duplicate scenario ID {}", scenario.id));
        }
        if scenario.interventions.is_empty() || scenario.interventions.len() > MAX_INTERVENTIONS {
            errors.push(format!(
                "{}: intervention count must be between 1 and {MAX_INTERVENTIONS}",
                scenario.id
            ));
        }
        let mut ids = BTreeSet::new();
        for intervention in &scenario.interventions {
            if !ids.insert(intervention.id.clone()) {
                errors.push(format!(
                    "{}: duplicate intervention ID {}",
                    scenario.id, intervention.id
                ));
            }
            if intervention.case_id.trim().is_empty() {
                errors.push(format!(
                    "{} / {}: case_id cannot be empty",
                    scenario.id, intervention.id
                ));
            }
            if intervention
                .requires_selected_interventions
                .contains(&intervention.id)
            {
                errors.push(format!(
                    "{} / {}: self-dependency is forbidden",
                    scenario.id, intervention.id
                ));
            }
        }
        for intervention in &scenario.interventions {
            for required in &intervention.requires_selected_interventions {
                if !ids.contains(required) {
                    errors.push(format!(
                        "{} / {}: dependency {} is not in the scenario",
                        scenario.id, intervention.id, required
                    ));
                }
            }
        }
        if scenario.expected_status == "feasible" {
            if scenario.expected_frontier.is_empty()
                || scenario.expected_feasible_portfolio_count == 0
                || !scenario.expected_infeasibility_codes.is_empty()
            {
                errors.push(format!(
                    "{}: feasible reference must have a frontier/count and no infeasibility codes",
                    scenario.id
                ));
            }
        } else if !scenario.expected_frontier.is_empty()
            || scenario.expected_feasible_portfolio_count != 0
            || scenario.expected_infeasibility_codes.is_empty()
        {
            errors.push(format!(
                "{}: infeasible reference must have no frontier/count and must explain blockers",
                scenario.id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    for scenario in &mut corpus.scenarios {
        scenario.interventions.sort_by(|a, b| a.id.cmp(&b.id));
        for intervention in &mut scenario.interventions {
            intervention.requires_selected_interventions.sort();
        }
        scenario.expected_infeasibility_codes.sort();
        for portfolio in &mut scenario.expected_frontier {
            portfolio.selected_intervention_ids.sort();
        }
        scenario.expected_frontier.sort_by(|a, b| {
            a.selected_intervention_ids
                .cmp(&b.selected_intervention_ids)
        });
    }
}

fn selected_is_structurally_valid(selected: &[&Intervention]) -> bool {
    if selected
        .iter()
        .any(|intervention| !intervention.selectable())
    {
        return false;
    }
    let mut case_ids = BTreeSet::new();
    for intervention in selected {
        if !case_ids.insert(intervention.case_id.as_str()) {
            return false;
        }
    }
    let selected_ids: BTreeSet<&str> = selected
        .iter()
        .map(|intervention| intervention.id.as_str())
        .collect();
    selected.iter().all(|intervention| {
        intervention
            .requires_selected_interventions
            .iter()
            .all(|required| selected_ids.contains(required.as_str()))
    })
}

fn compute_portfolio(
    scenario: &Scenario,
    selected: &[&Intervention],
) -> Result<PortfolioSummary, String> {
    let mut cost = PortfolioVector::default();
    let mut benefits = ServiceVector::default();
    let mut ids = Vec::new();

    for intervention in selected {
        cost = cost.checked_add(&intervention.resource_demand, &scenario.id)?;
        benefits.checked_add_service(
            &intervention.service_category,
            intervention.service_units,
            &scenario.id,
        )?;
        ids.push(intervention.id.clone());
    }
    ids.sort();

    Ok(PortfolioSummary {
        selected_intervention_ids: ids,
        service_benefits: benefits,
        resource_use: cost.as_resource_caps(),
        cost_vector: cost,
    })
}

fn portfolio_dominates(left: &PortfolioSummary, right: &PortfolioSummary) -> bool {
    let benefits_ok = left.service_benefits.dominates(&right.service_benefits);
    let costs_ok = left.cost_vector.componentwise_le(&right.cost_vector);
    let strict_benefit = left
        .service_benefits
        .strictly_better_than(&right.service_benefits);
    let strict_cost = left
        .cost_vector
        .componentwise_strictly_better(&right.cost_vector);
    benefits_ok && costs_ok && (strict_benefit || strict_cost)
}

fn same_objective_vector(left: &PortfolioSummary, right: &PortfolioSummary) -> bool {
    left.service_benefits == right.service_benefits && left.cost_vector == right.cost_vector
}

fn pareto_frontier(mut feasible: Vec<PortfolioSummary>) -> Vec<PortfolioSummary> {
    feasible.sort_by(|a, b| {
        a.selected_intervention_ids
            .cmp(&b.selected_intervention_ids)
    });
    let mut unique: Vec<PortfolioSummary> = Vec::new();
    for portfolio in feasible {
        if !unique
            .iter()
            .any(|existing| same_objective_vector(existing, &portfolio))
        {
            unique.push(portfolio);
        }
    }

    let frontier: Vec<PortfolioSummary> = unique
        .iter()
        .filter(|candidate| {
            !unique.iter().any(|other| {
                other.selected_intervention_ids != candidate.selected_intervention_ids
                    && portfolio_dominates(other, candidate)
            })
        })
        .cloned()
        .collect();
    let mut frontier = frontier;
    frontier.sort_by(|a, b| {
        a.selected_intervention_ids
            .cmp(&b.selected_intervention_ids)
    });
    frontier
}

fn solve_scenario(
    scenario: &Scenario,
) -> Result<(String, Vec<String>, usize, Vec<PortfolioSummary>), String> {
    if scenario.interventions.len() > MAX_INTERVENTIONS {
        return Err(format!(
            "{}: oracle_limit_exceeded; {} interventions exceeds cap {}",
            scenario.id,
            scenario.interventions.len(),
            MAX_INTERVENTIONS
        ));
    }
    let subset_count = 1usize
        .checked_shl(scenario.interventions.len() as u32)
        .ok_or_else(|| {
            format!(
                "{}: oracle_limit_exceeded while computing subsets",
                scenario.id
            )
        })?;
    if subset_count > MAX_SUBSETS {
        return Err(format!(
            "{}: oracle_limit_exceeded: {} subsets exceeds cap {}",
            scenario.id, subset_count, MAX_SUBSETS
        ));
    }

    let mut feasible_count = 0usize;
    let mut feasible_portfolios = Vec::new();
    let mut floor_achievable = false;
    let mut resource_blocked = false;
    let mut worker_burden_blocked = false;

    for mask in 0..subset_count {
        let selected: Vec<&Intervention> = scenario
            .interventions
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1usize << index) != 0)
            .map(|(_, intervention)| intervention)
            .collect();

        if !selected_is_structurally_valid(&selected) {
            continue;
        }

        let portfolio = compute_portfolio(scenario, &selected)?;
        if !portfolio.service_benefits.meets(&scenario.service_floors) {
            continue;
        }
        floor_achievable = true;

        let resources_fit = portfolio.cost_vector.fits(&scenario.resource_caps);
        let worker_burden_fits = scenario
            .worker_burden_cap_hours
            .is_none_or(|cap| portfolio.cost_vector.worker_burden_hours <= cap);
        if !resources_fit {
            resource_blocked = true;
        }
        if !worker_burden_fits {
            worker_burden_blocked = true;
        }

        if resources_fit && worker_burden_fits {
            feasible_count += 1;
            feasible_portfolios.push(portfolio);
        }
    }

    if feasible_count > 0 {
        return Ok((
            "feasible".to_string(),
            Vec::new(),
            feasible_count,
            pareto_frontier(feasible_portfolios),
        ));
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
    if worker_burden_blocked {
        codes.push("worker_burden_cap_infeasible".to_string());
    }
    codes.sort();

    let status = match (resource_blocked, worker_burden_blocked) {
        (true, true) => "infeasible_resource_and_worker_burden",
        (true, false) => "infeasible_resource_capacity",
        (false, true) => "infeasible_worker_burden",
        (false, false) => {
            return Err(format!(
                "{}: infeasibility classification has no blocking constraint",
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
        .map_err(|error| format!("portfolio corpus type decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let mut receipts = Vec::new();
    for scenario in &corpus.scenarios {
        let (status, mut codes, feasible_count, frontier) = solve_scenario(scenario)?;
        codes.sort();
        let passed = status == scenario.expected_status
            && codes == scenario.expected_infeasibility_codes
            && feasible_count == scenario.expected_feasible_portfolio_count
            && frontier == scenario.expected_frontier;
        receipts.push(ScenarioReceipt {
            scenario_id: scenario.id.clone(),
            status,
            infeasibility_codes: codes,
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
            "SYNTHETIC_PORTFOLIO_ORACLE_PASS"
        } else {
            "SYNTHETIC_PORTFOLIO_ORACLE_FAIL"
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
            "All interventions, capacities, costs and service units are synthetic contract inputs.",
            "The solver is an exact subset/Pareto oracle for small cases, not a production scheduler.",
            "A successful fixture receipt validates consistency with hand-enumerated cases only, not real service outcomes or job creation.",
            "Resource and worker constraints are hard gates; the solver never trades away an essential floor.",
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
        return Err(format!(
            "{} portfolio scenarios disagree with the frozen contract",
            report.scenarios_failed
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

    const FIXTURE: &str =
        include_str!("../../../docs/research/fixtures/workforce_portfolio_scenarios_v0_1.json");
    const SCHEMA: &str =
        include_str!("../../../docs/research/schemas/workforce_portfolio_v0_1.schema.json");

    fn loaded() -> (Value, Value) {
        (
            serde_json::from_str(FIXTURE).expect("fixture parses"),
            serde_json::from_str(SCHEMA).expect("schema parses"),
        )
    }

    fn report(fixture: &Value, schema: &Value) -> Report {
        let bytes = serde_json::to_vec(fixture).expect("fixture serializes");
        let schema_bytes = serde_json::to_vec(schema).expect("schema serializes");
        verify_values(schema, fixture, &bytes, &schema_bytes).expect("portfolio fixture verifies")
    }

    fn find<'a>(receipt: &'a Report, id: &str) -> &'a ScenarioReceipt {
        receipt
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == id)
            .expect("scenario exists")
    }

    #[test]
    fn exact_oracle_matches_all_frozen_frontiers_and_feasibility_states() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(report.status, "SYNTHETIC_PORTFOLIO_ORACLE_PASS");
        assert_eq!(report.scenario_count, 5);
        assert_eq!(report.scenarios_passed, 5);
        assert_eq!(report.scenarios_failed, 0);
    }

    #[test]
    fn shared_toolkit_and_shift_caps_leave_three_nondominated_portfolios() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let scenario = find(&report, "PF-001");
        assert_eq!(scenario.feasible_portfolio_count, 3);
        assert_eq!(scenario.frontier.len(), 3);
        assert!(!scenario.frontier.iter().any(|portfolio| {
            portfolio
                .selected_intervention_ids
                .contains(&"HEALTH_UNCERTAIN".to_string())
        }));
    }

    #[test]
    fn essential_service_floor_is_not_relaxed_for_an_ineligible_bundle() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let scenario = find(&report, "PF-002");
        assert_eq!(scenario.status, "infeasible_essential_floor");
        assert!(scenario.frontier.is_empty());
        assert_eq!(
            scenario.infeasibility_codes,
            vec!["essential_service_floor_unmet"]
        );
    }

    #[test]
    fn shared_capacity_and_worker_burden_are_hard_infeasibility_gates() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        assert_eq!(
            find(&report, "PF-003").status,
            "infeasible_resource_capacity"
        );
        assert_eq!(find(&report, "PF-004").status, "infeasible_worker_burden");
        assert!(find(&report, "PF-003").frontier.is_empty());
        assert!(find(&report, "PF-004").frontier.is_empty());
    }

    #[test]
    fn required_bundle_dependencies_are_enforced() {
        let (fixture, schema) = loaded();
        let report = report(&fixture, &schema);
        let scenario = find(&report, "PF-005");
        assert_eq!(scenario.feasible_portfolio_count, 1);
        assert_eq!(
            scenario.frontier[0].selected_intervention_ids,
            vec![
                "INSPECT_WATER".to_string(),
                "WATER_AFTER_INSPECTION".to_string()
            ]
        );
        assert!(
            !scenario.frontier[0]
                .selected_intervention_ids
                .contains(&"WATER_UNFUNDED".to_string())
        );
    }

    #[test]
    fn input_row_permutation_preserves_frontier_and_verdict() {
        let (fixture, schema) = loaded();
        let original = report(&fixture, &schema);
        let mut permuted = fixture.clone();
        permuted["scenarios"]
            .as_array_mut()
            .expect("scenarios")
            .reverse();
        for scenario in permuted["scenarios"].as_array_mut().expect("scenarios") {
            scenario["interventions"]
                .as_array_mut()
                .expect("interventions")
                .reverse();
            for intervention in scenario["interventions"]
                .as_array_mut()
                .expect("interventions")
            {
                intervention["requires_selected_interventions"]
                    .as_array_mut()
                    .expect("dependencies")
                    .reverse();
            }
        }
        let reordered = report(&permuted, &schema);
        assert_eq!(original.scenarios, reordered.scenarios);
    }

    #[test]
    fn unknown_dependencies_fail_closed() {
        let (mut fixture, schema) = loaded();
        fixture["scenarios"][4]["interventions"][1]["requires_selected_interventions"] =
            json!(["DOES_NOT_EXIST"]);
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
}
