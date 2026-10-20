use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_EXACT_PROJECTS: usize = 20;
const MAX_SEARCH_PROJECTS: usize = 63;
const OBJECTIVE_IDS: [&str; 4] = [
    "economic_value_u",
    "essential_service_points",
    "paid_work_opportunity_points",
    "ecological_quality_points",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveVector {
    pub economic_value_u: i64,
    pub essential_service_points: i64,
    pub paid_work_opportunity_points: i64,
    pub ecological_quality_points: i64,
}

impl ObjectiveVector {
    fn zero() -> Self {
        Self {
            economic_value_u: 0,
            essential_service_points: 0,
            paid_work_opportunity_points: 0,
            ecological_quality_points: 0,
        }
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        Some(Self {
            economic_value_u: self.economic_value_u.checked_add(rhs.economic_value_u)?,
            essential_service_points: self
                .essential_service_points
                .checked_add(rhs.essential_service_points)?,
            paid_work_opportunity_points: self
                .paid_work_opportunity_points
                .checked_add(rhs.paid_work_opportunity_points)?,
            ecological_quality_points: self
                .ecological_quality_points
                .checked_add(rhs.ecological_quality_points)?,
        })
    }

    fn value(&self, objective_id: &str) -> Option<i64> {
        match objective_id {
            "economic_value_u" => Some(self.economic_value_u),
            "essential_service_points" => Some(self.essential_service_points),
            "paid_work_opportunity_points" => Some(self.paid_work_opportunity_points),
            "ecological_quality_points" => Some(self.ecological_quality_points),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: String,
    pub weight: Option<f64>,
    pub probability: Option<f64>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodBudget {
    pub period: String,
    pub cost_cap_u: i64,
    pub crew_cap_u: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveDefinition {
    pub id: String,
    pub direction: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    pub id: String,
    pub metric: String,
    pub direction: String,
    pub threshold: i64,
    pub apply_to_each_scenario: bool,
    pub source_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub period: String,
    pub stage_order: u32,
    pub cost_u: i64,
    pub crew_u: i64,
    pub requires: Vec<String>,
    pub gate: String,
    pub outcomes: BTreeMap<String, ObjectiveVector>,
    pub evidence_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InteractionRule {
    pub id: String,
    pub kind: String,
    pub when_all_selected: Vec<String>,
    pub objective_adjustment_by_scenario: BTreeMap<String, ObjectiveVector>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumerationConfig {
    pub method: String,
    pub candidate_subset_count: u64,
    pub over_cap_behavior: String,
    pub objective_comparison: String,
    pub canonicalization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub schema_version: String,
    pub evidence_class: String,
    pub synthetic_only: bool,
    pub claim_ceiling: String,
    pub budgets_by_period: Vec<PeriodBudget>,
    pub scenarios: Vec<Scenario>,
    pub service_baseline_by_scenario: BTreeMap<String, i64>,
    pub mandatory_floors: Vec<Floor>,
    pub objective_schema: Vec<ObjectiveDefinition>,
    pub projects: Vec<Project>,
    pub interaction_rules: Vec<InteractionRule>,
    pub enumeration: EnumerationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub cost_u: i64,
    pub crew_u: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Portfolio {
    pub project_ids: Vec<String>,
    pub period_cost_and_crew: BTreeMap<String, Resources>,
    pub outcomes: BTreeMap<String, ObjectiveVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct RejectionCounts {
    pub gate_not_pass: u64,
    pub missing_prerequisite: u64,
    pub period_budget_exceeded: u64,
    pub crew_capacity_exceeded: u64,
    pub protected_service_floor: u64,
}

impl RejectionCounts {
    fn record(&mut self, reason: RejectionReason) {
        match reason {
            RejectionReason::GateNotPass => self.gate_not_pass += 1,
            RejectionReason::MissingPrerequisite => self.missing_prerequisite += 1,
            RejectionReason::PeriodBudgetExceeded => self.period_budget_exceeded += 1,
            RejectionReason::CrewCapacityExceeded => self.crew_capacity_exceeded += 1,
            RejectionReason::ProtectedServiceFloor => self.protected_service_floor += 1,
        }
    }

    pub fn total(&self) -> u64 {
        self.gate_not_pass
            + self.missing_prerequisite
            + self.period_budget_exceeded
            + self.crew_capacity_exceeded
            + self.protected_service_floor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RejectionReason {
    GateNotPass,
    MissingPrerequisite,
    PeriodBudgetExceeded,
    CrewCapacityExceeded,
    ProtectedServiceFloor,
}

enum SubsetEvaluation {
    Feasible(Portfolio),
    Rejected(RejectionReason),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evaluation {
    pub candidate_subset_count: u64,
    pub feasible_portfolios: Vec<Portfolio>,
    pub pareto_frontier: Vec<Portfolio>,
    pub rejection_reason_counts: RejectionCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct FrontierSearchStats {
    pub candidate_subset_count: u64,
    pub visited_nodes: u64,
    pub evaluated_leaves: u64,
    pub feasible_portfolio_count: u64,
    pub service_floor_rejection_leaves: u64,
    pub gate_pruned_subsets: u64,
    pub prerequisite_pruned_subsets: u64,
    pub budget_pruned_subsets: u64,
    pub crew_pruned_subsets: u64,
    pub frontier_comparisons: u64,
}

impl FrontierSearchStats {
    pub fn pruned_subset_count(&self) -> u64 {
        self.gate_pruned_subsets
            + self.prerequisite_pruned_subsets
            + self.budget_pruned_subsets
            + self.crew_pruned_subsets
    }

    pub fn accounted_subset_count(&self) -> u64 {
        self.evaluated_leaves + self.pruned_subset_count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontierSearchEvaluation {
    pub stats: FrontierSearchStats,
    pub pareto_frontier: Vec<Portfolio>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioOmissionPortfolioReport {
    pub project_ids: Vec<String>,
    pub full_reference_feasible: bool,
    pub reference_outcomes_by_scenario: Option<BTreeMap<String, ObjectiveVector>>,
    pub regret_by_scenario: Option<BTreeMap<String, ObjectiveVector>>,
    pub worst_case_regret: Option<ObjectiveVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Complete, explicit payload for a scenario withheld from the selection input.
/// Maps must cover every current project and every current interaction rule exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioPatch {
    pub scenario: Scenario,
    pub service_baseline: i64,
    pub project_outcomes_by_id: BTreeMap<String, ObjectiveVector>,
    pub interaction_adjustments_by_rule_id: BTreeMap<String, ObjectiveVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioAdditionReport {
    pub added_scenario_id: String,
    pub original_scenario_ids: Vec<String>,
    pub augmented_scenario_ids: Vec<String>,
    pub candidate_subset_count: u64,
    pub original_feasible_portfolio_count: usize,
    pub augmented_feasible_portfolio_count: usize,
    pub original_frontier_project_ids: Vec<Vec<String>>,
    pub augmented_frontier_project_ids: Vec<Vec<String>>,
    pub original_feasible_removed_by_addition: Vec<Vec<String>>,
    pub frontier_id_intersection_count: usize,
    pub frontier_id_union_count: usize,
}

pub struct ScenarioOmissionReport {
    pub omitted_scenario_id: String,
    pub original_scenario_ids: Vec<String>,
    pub reduced_scenario_ids: Vec<String>,
    pub candidate_subset_count: u64,
    pub reduced_feasible_portfolio_count: usize,
    pub reference_frontier_project_ids: Vec<Vec<String>>,
    pub reduced_frontier: Vec<ScenarioOmissionPortfolioReport>,
    pub frontier_id_intersection_count: usize,
    pub frontier_id_union_count: usize,
    pub full_reference_infeasible_frontier_count: usize,
}

fn period_rank(period: &str) -> Option<u32> {
    period.strip_prefix('P')?.parse::<u32>().ok()
}

fn is_earlier(prerequisite: &Project, dependent: &Project) -> bool {
    match (
        period_rank(&prerequisite.period),
        period_rank(&dependent.period),
    ) {
        (Some(a), Some(b)) if a < b => true,
        (Some(a), Some(b)) if a == b => prerequisite.stage_order < dependent.stage_order,
        _ => false,
    }
}

fn portfolio_key(portfolio: &Portfolio) -> String {
    portfolio.project_ids.join("+")
}

fn validate_input(input: &Input) -> Result<(), String> {
    validate_input_with_limit(input, MAX_EXACT_PROJECTS)
}

fn validate_search_input(input: &Input) -> Result<(), String> {
    validate_input_with_limit(input, MAX_SEARCH_PROJECTS)
}

fn validate_input_with_limit(input: &Input, max_projects: usize) -> Result<(), String> {
    let v01 = input.schema_version == "sa-economic-portfolio-v0.1";
    let v02 = input.schema_version == "sa-economic-portfolio-v0.2";
    if (!v01 && !v02) || input.evidence_class != "synthetic" || !input.synthetic_only {
        return Err("unsupported schema or non-synthetic input".to_owned());
    }

    if input.projects.is_empty() || input.projects.len() > max_projects {
        return Err(format!(
            "portfolio evaluator accepts 1..={max_projects} projects; got {}",
            input.projects.len()
        ));
    }
    let subset_count = 1_u64
        .checked_shl(input.projects.len() as u32)
        .ok_or_else(|| "project subset count overflow".to_owned())?;
    if input.enumeration.candidate_subset_count != subset_count {
        return Err(format!(
            "declared subset count {} does not equal exhaustive count {subset_count}",
            input.enumeration.candidate_subset_count
        ));
    }

    if input.enumeration.method != "complete subset enumeration"
        || input.enumeration.over_cap_behavior
            != "structured error; never return a partial frontier as complete"
        || input.enumeration.objective_comparison
            != "scenario-wise componentwise maximization over every declared scenario and objective; scenario cases have no probabilities"
        || input.enumeration.canonicalization
            != "sort project IDs within each portfolio and sort portfolio ID tuples lexicographically"
    {
        return Err("unsupported enumeration contract".to_owned());
    }

    let scenario_ids: BTreeSet<String> = input
        .scenarios
        .iter()
        .map(|scenario| scenario.id.clone())
        .collect();
    let v01_scenarios = BTreeSet::from(["nominal".to_owned(), "hot_dry_grid_stress".to_owned()]);
    if scenario_ids.is_empty()
        || scenario_ids.len() != input.scenarios.len()
        || scenario_ids.iter().any(String::is_empty)
        || (v01 && scenario_ids != v01_scenarios)
    {
        return Err(
            "scenario IDs must be unique, non-empty, and match the v0.1 set when applicable"
                .to_owned(),
        );
    }
    if input
        .scenarios
        .iter()
        .any(|scenario| scenario.weight.is_some() || scenario.probability.is_some())
    {
        return Err(
            "synthetic stress scenarios must not carry uncalibrated weights/probabilities"
                .to_owned(),
        );
    }

    let budget_periods: BTreeSet<String> = input
        .budgets_by_period
        .iter()
        .map(|budget| budget.period.clone())
        .collect();
    let period_ranks: BTreeSet<u32> = budget_periods
        .iter()
        .filter_map(|period| period_rank(period))
        .collect();
    let v01_periods = BTreeSet::from(["P1".to_owned(), "P2".to_owned()]);
    if budget_periods.is_empty()
        || budget_periods.len() != input.budgets_by_period.len()
        || budget_periods
            .iter()
            .any(|period| !matches!(period_rank(period), Some(rank) if rank > 0))
        || period_ranks.len() != budget_periods.len()
        || (v01 && budget_periods != v01_periods)
    {
        return Err(
            "period budgets must be unique positive Pn periods and match P1/P2 for v0.1".to_owned(),
        );
    }
    if input
        .budgets_by_period
        .iter()
        .any(|budget| budget.cost_cap_u < 0 || budget.crew_cap_u < 0)
    {
        return Err("negative budget or crew cap".to_owned());
    }

    let expected_objectives: BTreeSet<String> =
        OBJECTIVE_IDS.iter().map(|id| (*id).to_owned()).collect();
    let objective_ids: BTreeSet<String> = input
        .objective_schema
        .iter()
        .map(|objective| objective.id.clone())
        .collect();
    if objective_ids.len() != input.objective_schema.len()
        || objective_ids != expected_objectives
        || input
            .objective_schema
            .iter()
            .any(|objective| objective.direction != "maximize" || objective.unit.is_empty())
    {
        return Err(
            "objective schema must declare the four unique v0.1/v0.2 maximization objectives"
                .to_owned(),
        );
    }

    if input.mandatory_floors.is_empty()
        || input.mandatory_floors.iter().any(|floor| {
            floor.metric != "total_essential_service_points"
                || floor.direction != "minimize violation"
                || !floor.apply_to_each_scenario
                || floor.source_class != "synthetic test invariant"
        })
    {
        return Err("unsupported or incomplete mandatory-floor contract".to_owned());
    }

    if input
        .service_baseline_by_scenario
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>()
        != scenario_ids
    {
        return Err("service baseline must cover exactly the declared scenarios".to_owned());
    }

    let project_ids: BTreeSet<String> = input
        .projects
        .iter()
        .map(|project| project.id.clone())
        .collect();
    let v01_project_ids = BTreeSet::from([
        "T".to_owned(),
        "W".to_owned(),
        "A".to_owned(),
        "I".to_owned(),
        "R".to_owned(),
        "J".to_owned(),
        "F".to_owned(),
        "X".to_owned(),
    ]);
    if project_ids.len() != input.projects.len()
        || project_ids.iter().any(String::is_empty)
        || (v01 && project_ids != v01_project_ids)
    {
        return Err(
            "project IDs must be unique/non-empty and match the frozen v0.1 corpus when applicable"
                .to_owned(),
        );
    }

    let by_id: BTreeMap<&str, &Project> = input
        .projects
        .iter()
        .map(|project| (project.id.as_str(), project))
        .collect();
    for project in &input.projects {
        if !budget_periods.contains(&project.period)
            || period_rank(&project.period).is_none()
            || project.stage_order == 0
            || project.cost_u < 0
            || project.crew_u < 0
            || !matches!(project.gate.as_str(), "PASS" | "BLOCK" | "UNKNOWN")
            || !matches!(
                project.evidence_class.as_str(),
                "synthetic" | "synthetic hostile negative control"
            )
        {
            return Err(format!("invalid project metadata for {}", project.id));
        }
        if project.outcomes.keys().cloned().collect::<BTreeSet<_>>() != scenario_ids {
            return Err(format!(
                "scenario coverage mismatch for project {}",
                project.id
            ));
        }
        let mut local_dependencies = BTreeSet::new();
        for dependency_id in &project.requires {
            if !local_dependencies.insert(dependency_id) {
                return Err(format!("duplicate prerequisite on project {}", project.id));
            }
            let dependency = by_id
                .get(dependency_id.as_str())
                .ok_or_else(|| format!("unknown prerequisite {dependency_id}"))?;
            if !is_earlier(dependency, project) {
                return Err(format!(
                    "prerequisite {dependency_id} is not staged before dependent {}",
                    project.id
                ));
            }
        }
    }

    let mut rule_ids = BTreeSet::new();
    for rule in &input.interaction_rules {
        if !rule_ids.insert(rule.id.as_str()) || rule.when_all_selected.len() < 2 {
            return Err("duplicate interaction ID or too-small interaction set".to_owned());
        }
        if !matches!(rule.kind.as_str(), "synergy" | "overlap_adjustment") {
            return Err(format!("unsupported interaction kind in {}", rule.id));
        }
        let participants: BTreeSet<&str> =
            rule.when_all_selected.iter().map(String::as_str).collect();
        if participants.len() != rule.when_all_selected.len()
            || participants.iter().any(|id| !by_id.contains_key(*id))
        {
            return Err(format!("invalid participants for interaction {}", rule.id));
        }
        if rule
            .objective_adjustment_by_scenario
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != scenario_ids
        {
            return Err(format!(
                "scenario coverage mismatch for interaction {}",
                rule.id
            ));
        }
    }

    if input
        .service_baseline_by_scenario
        .values()
        .any(|value| *value < 0)
    {
        return Err("negative baseline service value".to_owned());
    }
    Ok(())
}

fn evaluate_subset(input: &Input, mask: u64) -> Result<SubsetEvaluation, String> {
    let selected: Vec<&Project> = input
        .projects
        .iter()
        .enumerate()
        .filter_map(|(index, project)| ((mask & (1_u64 << index)) != 0).then_some(project))
        .collect();

    if selected.iter().any(|project| project.gate != "PASS") {
        return Ok(SubsetEvaluation::Rejected(RejectionReason::GateNotPass));
    }

    let selected_ids: BTreeSet<&str> = selected.iter().map(|project| project.id.as_str()).collect();
    let by_id: BTreeMap<&str, &Project> = input
        .projects
        .iter()
        .map(|project| (project.id.as_str(), project))
        .collect();
    for project in &selected {
        for dependency_id in &project.requires {
            if !selected_ids.contains(dependency_id.as_str()) {
                return Ok(SubsetEvaluation::Rejected(
                    RejectionReason::MissingPrerequisite,
                ));
            }
            let dependency = by_id
                .get(dependency_id.as_str())
                .ok_or_else(|| format!("unknown prerequisite {dependency_id}"))?;
            if !is_earlier(dependency, project) {
                return Err(format!(
                    "prerequisite {dependency_id} is not staged before dependent {}",
                    project.id
                ));
            }
        }
    }

    let mut period_cost_and_crew = BTreeMap::new();
    for budget in &input.budgets_by_period {
        let mut used = Resources {
            cost_u: 0,
            crew_u: 0,
        };
        for project in selected
            .iter()
            .filter(|project| project.period.as_str() == budget.period.as_str())
        {
            used.cost_u = used
                .cost_u
                .checked_add(project.cost_u)
                .ok_or_else(|| "cost total overflow".to_owned())?;
            used.crew_u = used
                .crew_u
                .checked_add(project.crew_u)
                .ok_or_else(|| "crew total overflow".to_owned())?;
        }
        if used.cost_u > budget.cost_cap_u {
            return Ok(SubsetEvaluation::Rejected(
                RejectionReason::PeriodBudgetExceeded,
            ));
        }
        if used.crew_u > budget.crew_cap_u {
            return Ok(SubsetEvaluation::Rejected(
                RejectionReason::CrewCapacityExceeded,
            ));
        }
        period_cost_and_crew.insert(budget.period.clone(), used);
    }

    let mut outcomes = BTreeMap::new();
    for scenario in &input.scenarios {
        let baseline = input.service_baseline_by_scenario[&scenario.id];
        let mut total = ObjectiveVector {
            essential_service_points: baseline,
            ..ObjectiveVector::zero()
        };
        for project in &selected {
            total = total
                .checked_add(&project.outcomes[&scenario.id])
                .ok_or_else(|| format!("objective overflow in scenario {}", scenario.id))?;
        }

        for rule in &input.interaction_rules {
            if rule
                .when_all_selected
                .iter()
                .all(|id| selected_ids.contains(id.as_str()))
            {
                total = total
                    .checked_add(&rule.objective_adjustment_by_scenario[&scenario.id])
                    .ok_or_else(|| format!("interaction overflow in scenario {}", scenario.id))?;
            }
        }

        for floor in &input.mandatory_floors {
            let measured = match floor.metric.as_str() {
                "total_essential_service_points" => total.essential_service_points,
                _ => return Err(format!("unsupported floor metric {}", floor.metric)),
            };
            if measured < floor.threshold {
                return Ok(SubsetEvaluation::Rejected(
                    RejectionReason::ProtectedServiceFloor,
                ));
            }
        }
        outcomes.insert(scenario.id.clone(), total);
    }

    let mut project_ids: Vec<String> = selected.iter().map(|project| project.id.clone()).collect();
    project_ids.sort();
    Ok(SubsetEvaluation::Feasible(Portfolio {
        project_ids,
        period_cost_and_crew,
        outcomes,
    }))
}

fn dominates(candidate: &Portfolio, other: &Portfolio, scenarios: &[Scenario]) -> bool {
    let mut strictly_better = false;
    for scenario in scenarios {
        let Some(candidate_values) = candidate.outcomes.get(&scenario.id) else {
            return false;
        };
        let Some(other_values) = other.outcomes.get(&scenario.id) else {
            return false;
        };
        for objective_id in OBJECTIVE_IDS {
            let Some(a) = candidate_values.value(objective_id) else {
                return false;
            };
            let Some(b) = other_values.value(objective_id) else {
                return false;
            };
            if a < b {
                return false;
            }
            if a > b {
                strictly_better = true;
            }
        }
    }
    strictly_better
}

pub fn evaluate(input: &Input) -> Result<Evaluation, String> {
    validate_input(input)?;
    let candidate_subset_count = 1_u64
        .checked_shl(input.projects.len() as u32)
        .ok_or_else(|| "project subset count overflow".to_owned())?;

    let mut feasible_portfolios = Vec::new();
    let mut rejection_reason_counts = RejectionCounts::default();
    for mask in 0..candidate_subset_count {
        match evaluate_subset(input, mask)? {
            SubsetEvaluation::Feasible(portfolio) => feasible_portfolios.push(portfolio),
            SubsetEvaluation::Rejected(reason) => rejection_reason_counts.record(reason),
        }
    }
    feasible_portfolios.sort_by_key(portfolio_key);
    if feasible_portfolios.len() as u64 + rejection_reason_counts.total() != candidate_subset_count
    {
        return Err("internal accounting error: subset dispositions do not reconcile".to_owned());
    }

    let mut pareto_frontier = Vec::new();
    for (candidate_index, candidate) in feasible_portfolios.iter().enumerate() {
        let dominated = feasible_portfolios
            .iter()
            .enumerate()
            .any(|(other_index, other)| {
                candidate_index != other_index && dominates(other, candidate, &input.scenarios)
            });
        if !dominated {
            pareto_frontier.push(candidate.clone());
        }
    }
    pareto_frontier.sort_by_key(portfolio_key);

    Ok(Evaluation {
        candidate_subset_count,
        feasible_portfolios,
        pareto_frontier,
        rejection_reason_counts,
    })
}

struct FrontierSearchContext<'a> {
    input: &'a Input,
    project_order: Vec<usize>,
    project_index_by_id: BTreeMap<String, usize>,
    budgets_by_period: BTreeMap<String, PeriodBudget>,
}

#[derive(Default)]
struct FrontierSearchAccumulator {
    stats: FrontierSearchStats,
    pareto_frontier: Vec<Portfolio>,
}

fn insert_frontier_candidate(
    accumulator: &mut FrontierSearchAccumulator,
    candidate: Portfolio,
    scenarios: &[Scenario],
) {
    for existing in &accumulator.pareto_frontier {
        accumulator.stats.frontier_comparisons += 1;
        if dominates(existing, &candidate, scenarios) {
            return;
        }
    }

    let mut retained = Vec::with_capacity(accumulator.pareto_frontier.len() + 1);
    for existing in accumulator.pareto_frontier.drain(..) {
        accumulator.stats.frontier_comparisons += 1;
        if !dominates(&candidate, &existing, scenarios) {
            retained.push(existing);
        }
    }
    retained.push(candidate);
    accumulator.pareto_frontier = retained;
}

fn search_frontier_recursive(
    context: &FrontierSearchContext<'_>,
    position: usize,
    mask: u64,
    used_resources: &mut BTreeMap<String, Resources>,
    accumulator: &mut FrontierSearchAccumulator,
) -> Result<(), String> {
    accumulator.stats.visited_nodes += 1;
    if position == context.project_order.len() {
        accumulator.stats.evaluated_leaves += 1;
        match evaluate_subset(context.input, mask)? {
            SubsetEvaluation::Feasible(portfolio) => {
                accumulator.stats.feasible_portfolio_count += 1;
                insert_frontier_candidate(accumulator, portfolio, &context.input.scenarios);
            }
            SubsetEvaluation::Rejected(RejectionReason::ProtectedServiceFloor) => {
                accumulator.stats.service_floor_rejection_leaves += 1;
            }
            SubsetEvaluation::Rejected(reason) => {
                return Err(format!(
                    "frontier search reached a leaf rejected by an unpruned constraint: {reason:?}"
                ));
            }
        }
        return Ok(());
    }

    let project_index = context.project_order[position];
    let project = &context.input.projects[project_index];
    let remaining = context.project_order.len() - position - 1;
    let completion_count = 1_u64
        .checked_shl(remaining as u32)
        .ok_or_else(|| "frontier search subtree count overflow".to_owned())?;

    // Excluding a project always remains a possible decision.
    search_frontier_recursive(context, position + 1, mask, used_resources, accumulator)?;

    // Each include branch is pruned only when every completion beneath it is infeasible.
    if project.gate != "PASS" {
        accumulator.stats.gate_pruned_subsets += completion_count;
        return Ok(());
    }

    for dependency_id in &project.requires {
        let dependency_index = context
            .project_index_by_id
            .get(dependency_id)
            .ok_or_else(|| format!("unknown prerequisite {dependency_id}"))?;
        if mask & (1_u64 << *dependency_index) == 0 {
            accumulator.stats.prerequisite_pruned_subsets += completion_count;
            return Ok(());
        }
    }

    let budget = context
        .budgets_by_period
        .get(&project.period)
        .ok_or_else(|| format!("missing budget for period {}", project.period))?;
    let previous = used_resources
        .get(&project.period)
        .ok_or_else(|| format!("missing resource accumulator for period {}", project.period))?
        .clone();
    let next_cost = previous
        .cost_u
        .checked_add(project.cost_u)
        .ok_or_else(|| "cost total overflow during frontier search".to_owned())?;
    let next_crew = previous
        .crew_u
        .checked_add(project.crew_u)
        .ok_or_else(|| "crew total overflow during frontier search".to_owned())?;
    if next_cost > budget.cost_cap_u {
        accumulator.stats.budget_pruned_subsets += completion_count;
        return Ok(());
    }
    if next_crew > budget.crew_cap_u {
        accumulator.stats.crew_pruned_subsets += completion_count;
        return Ok(());
    }

    used_resources.insert(
        project.period.clone(),
        Resources {
            cost_u: next_cost,
            crew_u: next_crew,
        },
    );
    search_frontier_recursive(
        context,
        position + 1,
        mask | (1_u64 << project_index),
        used_resources,
        accumulator,
    )?;
    used_resources.insert(project.period.clone(), previous);
    Ok(())
}

/// Return the exact scenario-wise Pareto frontier using dependency-aware recursive search.
///
/// This is a candidate for scale-up, not an approximation: every pruned subtree is excluded
/// only when all completions are infeasible. Worst-case time remains exponential. The method
/// accepts up to 63 synthetic projects so subset masks/counts fit in u64; this is not a promise
/// of practical runtime at that maximum.
pub fn evaluate_frontier_search(input: &Input) -> Result<FrontierSearchEvaluation, String> {
    validate_search_input(input)?;
    let candidate_subset_count = 1_u64
        .checked_shl(input.projects.len() as u32)
        .ok_or_else(|| "project subset count overflow".to_owned())?;

    let mut project_order: Vec<usize> = (0..input.projects.len()).collect();
    project_order.sort_by_key(|index| {
        let project = &input.projects[*index];
        (
            period_rank(&project.period).unwrap_or(u32::MAX),
            project.stage_order,
            project.id.clone(),
        )
    });

    let project_index_by_id: BTreeMap<String, usize> = input
        .projects
        .iter()
        .enumerate()
        .map(|(index, project)| (project.id.clone(), index))
        .collect();
    let budgets_by_period: BTreeMap<String, PeriodBudget> = input
        .budgets_by_period
        .iter()
        .map(|budget| (budget.period.clone(), budget.clone()))
        .collect();
    let context = FrontierSearchContext {
        input,
        project_order,
        project_index_by_id,
        budgets_by_period,
    };
    let mut used_resources: BTreeMap<String, Resources> = input
        .budgets_by_period
        .iter()
        .map(|budget| {
            (
                budget.period.clone(),
                Resources {
                    cost_u: 0,
                    crew_u: 0,
                },
            )
        })
        .collect();
    let mut accumulator = FrontierSearchAccumulator {
        stats: FrontierSearchStats {
            candidate_subset_count,
            ..FrontierSearchStats::default()
        },
        pareto_frontier: Vec::new(),
    };

    search_frontier_recursive(&context, 0, 0, &mut used_resources, &mut accumulator)?;
    accumulator.pareto_frontier.sort_by_key(portfolio_key);
    if accumulator.stats.accounted_subset_count() != candidate_subset_count {
        return Err("frontier search subset accounting does not reconcile".to_owned());
    }

    Ok(FrontierSearchEvaluation {
        stats: accumulator.stats,
        pareto_frontier: accumulator.pareto_frontier,
    })
}

/// Remove one declared scenario from a valid v0.2 synthetic input.
///
/// This is an input transformation for sensitivity analysis, not a claim that
/// the omitted scenario is implausible. It fails closed on v0.1 inputs,
/// unknown scenarios, missing scenario coefficients, and a final remaining
/// scenario. The source input is never mutated.
pub fn omit_scenario(input: &Input, scenario_id: &str) -> Result<Input, String> {
    validate_input(input)?;
    if input.schema_version != "sa-economic-portfolio-v0.2" {
        return Err("scenario omission requires a v0.2 input contract".to_owned());
    }
    if input.scenarios.len() <= 1 {
        return Err("cannot omit the only remaining scenario".to_owned());
    }
    if !input
        .scenarios
        .iter()
        .any(|scenario| scenario.id == scenario_id)
    {
        return Err(format!("unknown scenario ID: {scenario_id}"));
    }

    let mut omitted = input.clone();
    omitted
        .scenarios
        .retain(|scenario| scenario.id != scenario_id);
    omitted
        .service_baseline_by_scenario
        .remove(scenario_id)
        .ok_or_else(|| format!("missing service baseline for scenario: {scenario_id}"))?;

    for project in &mut omitted.projects {
        project.outcomes.remove(scenario_id).ok_or_else(|| {
            format!(
                "project {} has no outcome for omitted scenario {}",
                project.id, scenario_id
            )
        })?;
    }
    for rule in &mut omitted.interaction_rules {
        rule.objective_adjustment_by_scenario
            .remove(scenario_id)
            .ok_or_else(|| {
                format!(
                    "interaction rule {} has no adjustment for omitted scenario {}",
                    rule.id, scenario_id
                )
            })?;
    }

    validate_input(&omitted)?;
    Ok(omitted)
}

fn scenario_objective_ideal(
    feasible: &[Portfolio],
    scenario_id: &str,
) -> Result<ObjectiveVector, String> {
    let mut ideal: Option<ObjectiveVector> = None;
    for portfolio in feasible {
        let outcome = portfolio
            .outcomes
            .get(scenario_id)
            .ok_or_else(|| format!("portfolio lacks scenario outcome: {scenario_id}"))?;
        match ideal.as_mut() {
            None => ideal = Some(outcome.clone()),
            Some(current) => {
                current.economic_value_u = current.economic_value_u.max(outcome.economic_value_u);
                current.essential_service_points = current
                    .essential_service_points
                    .max(outcome.essential_service_points);
                current.paid_work_opportunity_points = current
                    .paid_work_opportunity_points
                    .max(outcome.paid_work_opportunity_points);
                current.ecological_quality_points = current
                    .ecological_quality_points
                    .max(outcome.ecological_quality_points);
            }
        }
    }
    ideal.ok_or_else(|| "cannot establish scenario ideals without feasible portfolios".to_owned())
}

fn objective_regret(
    ideal: &ObjectiveVector,
    outcome: &ObjectiveVector,
) -> Result<ObjectiveVector, String> {
    let subtract = |lhs: i64, rhs: i64, objective: &str| {
        lhs.checked_sub(rhs)
            .filter(|value| *value >= 0)
            .ok_or_else(|| format!("invalid or overflowing regret for objective {objective}"))
    };
    Ok(ObjectiveVector {
        economic_value_u: subtract(
            ideal.economic_value_u,
            outcome.economic_value_u,
            "economic_value_u",
        )?,
        essential_service_points: subtract(
            ideal.essential_service_points,
            outcome.essential_service_points,
            "essential_service_points",
        )?,
        paid_work_opportunity_points: subtract(
            ideal.paid_work_opportunity_points,
            outcome.paid_work_opportunity_points,
            "paid_work_opportunity_points",
        )?,
        ecological_quality_points: subtract(
            ideal.ecological_quality_points,
            outcome.ecological_quality_points,
            "ecological_quality_points",
        )?,
    })
}

/// Recompute the exact frontier after omitting one scenario, then rescore the
/// reduced frontier against the full reference input. Only v0.2 inputs within
/// the exact evaluator's cap are accepted so full-feasibility status is exact.
pub fn evaluate_scenario_omission(
    input: &Input,
    scenario_id: &str,
) -> Result<ScenarioOmissionReport, String> {
    let full = evaluate(input)?;
    if full.feasible_portfolios.is_empty() {
        return Err("full reference input has no feasible portfolios".to_owned());
    }
    let reduced_input = omit_scenario(input, scenario_id)?;
    let reduced = evaluate(&reduced_input)?;

    let mut original_scenario_ids: Vec<String> = input
        .scenarios
        .iter()
        .map(|scenario| scenario.id.clone())
        .collect();
    original_scenario_ids.sort();
    let mut reduced_scenario_ids: Vec<String> = reduced_input
        .scenarios
        .iter()
        .map(|scenario| scenario.id.clone())
        .collect();
    reduced_scenario_ids.sort();

    let mut ideals = BTreeMap::new();
    for scenario_id in &original_scenario_ids {
        ideals.insert(
            scenario_id.clone(),
            scenario_objective_ideal(&full.feasible_portfolios, scenario_id)?,
        );
    }

    let full_feasible_by_key: BTreeMap<String, &Portfolio> = full
        .feasible_portfolios
        .iter()
        .map(|portfolio| (portfolio_key(portfolio), portfolio))
        .collect();
    let reference_frontier_project_ids: Vec<Vec<String>> = full
        .pareto_frontier
        .iter()
        .map(|portfolio| portfolio.project_ids.clone())
        .collect();
    let reference_frontier_ids: BTreeSet<String> =
        full.pareto_frontier.iter().map(portfolio_key).collect();
    let reduced_frontier_ids: BTreeSet<String> =
        reduced.pareto_frontier.iter().map(portfolio_key).collect();
    let frontier_id_intersection_count = reference_frontier_ids
        .intersection(&reduced_frontier_ids)
        .count();
    let frontier_id_union_count = reference_frontier_ids.union(&reduced_frontier_ids).count();

    let mut full_reference_infeasible_frontier_count = 0;
    let mut reduced_frontier = Vec::with_capacity(reduced.pareto_frontier.len());
    for candidate in &reduced.pareto_frontier {
        let key = portfolio_key(candidate);
        let Some(reference_portfolio) = full_feasible_by_key.get(&key) else {
            full_reference_infeasible_frontier_count += 1;
            reduced_frontier.push(ScenarioOmissionPortfolioReport {
                project_ids: candidate.project_ids.clone(),
                full_reference_feasible: false,
                reference_outcomes_by_scenario: None,
                regret_by_scenario: None,
                worst_case_regret: None,
            });
            continue;
        };

        let reference_outcomes_by_scenario = reference_portfolio.outcomes.clone();
        let mut regret_by_scenario = BTreeMap::new();
        let mut worst_case_regret = ObjectiveVector::zero();
        for reference_scenario_id in &original_scenario_ids {
            let ideal = ideals
                .get(reference_scenario_id)
                .ok_or_else(|| format!("missing objective ideal: {reference_scenario_id}"))?;
            let outcome = reference_outcomes_by_scenario
                .get(reference_scenario_id)
                .ok_or_else(|| format!("missing rescored outcome: {reference_scenario_id}"))?;
            let regret = objective_regret(ideal, outcome)?;
            worst_case_regret.economic_value_u = worst_case_regret
                .economic_value_u
                .max(regret.economic_value_u);
            worst_case_regret.essential_service_points = worst_case_regret
                .essential_service_points
                .max(regret.essential_service_points);
            worst_case_regret.paid_work_opportunity_points = worst_case_regret
                .paid_work_opportunity_points
                .max(regret.paid_work_opportunity_points);
            worst_case_regret.ecological_quality_points = worst_case_regret
                .ecological_quality_points
                .max(regret.ecological_quality_points);
            regret_by_scenario.insert(reference_scenario_id.clone(), regret);
        }
        reduced_frontier.push(ScenarioOmissionPortfolioReport {
            project_ids: candidate.project_ids.clone(),
            full_reference_feasible: true,
            reference_outcomes_by_scenario: Some(reference_outcomes_by_scenario),
            regret_by_scenario: Some(regret_by_scenario),
            worst_case_regret: Some(worst_case_regret),
        });
    }

    Ok(ScenarioOmissionReport {
        omitted_scenario_id: scenario_id.to_owned(),
        original_scenario_ids,
        reduced_scenario_ids,
        candidate_subset_count: reduced.candidate_subset_count,
        reduced_feasible_portfolio_count: reduced.feasible_portfolios.len(),
        reference_frontier_project_ids,
        reduced_frontier,
        frontier_id_intersection_count,
        frontier_id_union_count,
        full_reference_infeasible_frontier_count,
    })
}

/// Add one complete, withheld scenario to a valid v0.2 synthetic input.
/// All project and interaction outcome rows must be supplied explicitly.
pub fn add_scenario(input: &Input, patch: &ScenarioPatch) -> Result<Input, String> {
    validate_input(input)?;
    if input.schema_version != "sa-economic-portfolio-v0.2" {
        return Err("scenario addition requires a v0.2 input contract".to_owned());
    }
    if patch.scenario.id.trim().is_empty()
        || patch.scenario.weight.is_some()
        || patch.scenario.probability.is_some()
    {
        return Err("scenario patch needs a non-empty ID and no uncalibrated probabilities".to_owned());
    }
    if input.scenarios.iter().any(|scenario| scenario.id == patch.scenario.id) {
        return Err(format!("scenario already exists: {}", patch.scenario.id));
    }
    if patch.service_baseline < 0 {
        return Err("scenario patch has a negative service baseline".to_owned());
    }

    let expected_project_ids: BTreeSet<String> =
        input.projects.iter().map(|project| project.id.clone()).collect();
    let supplied_project_ids: BTreeSet<String> =
        patch.project_outcomes_by_id.keys().cloned().collect();
    if supplied_project_ids != expected_project_ids {
        return Err("scenario patch must provide exactly one outcome for every project".to_owned());
    }

    let expected_rule_ids: BTreeSet<String> = input
        .interaction_rules
        .iter()
        .map(|rule| rule.id.clone())
        .collect();
    let supplied_rule_ids: BTreeSet<String> = patch
        .interaction_adjustments_by_rule_id
        .keys()
        .cloned()
        .collect();
    if supplied_rule_ids != expected_rule_ids {
        return Err(
            "scenario patch must provide exactly one adjustment for every interaction rule"
                .to_owned(),
        );
    }

    let mut augmented = input.clone();
    let scenario_id = patch.scenario.id.clone();
    augmented.scenarios.push(patch.scenario.clone());
    augmented
        .service_baseline_by_scenario
        .insert(scenario_id.clone(), patch.service_baseline);
    for project in &mut augmented.projects {
        let outcome = patch.project_outcomes_by_id.get(&project.id)
            .ok_or_else(|| format!("missing project outcome for {}", project.id))?;
        project.outcomes.insert(scenario_id.clone(), outcome.clone());
    }
    for rule in &mut augmented.interaction_rules {
        let adjustment = patch.interaction_adjustments_by_rule_id.get(&rule.id)
            .ok_or_else(|| format!("missing interaction adjustment for {}", rule.id))?;
        rule.objective_adjustment_by_scenario.insert(scenario_id.clone(), adjustment.clone());
    }

    validate_input(&augmented)?;
    Ok(augmented)
}

/// Add a withheld scenario and exactly compare its feasible set and frontier.
/// Adding scenario-wise hard floors may remove feasible portfolios but cannot add them.
pub fn evaluate_scenario_addition(
    input: &Input,
    patch: &ScenarioPatch,
) -> Result<ScenarioAdditionReport, String> {
    let original = evaluate(input)?;
    let augmented_input = add_scenario(input, patch)?;
    let augmented = evaluate(&augmented_input)?;

    let mut original_scenario_ids: Vec<String> =
        input.scenarios.iter().map(|scenario| scenario.id.clone()).collect();
    original_scenario_ids.sort();
    let mut augmented_scenario_ids: Vec<String> = augmented_input
        .scenarios.iter().map(|scenario| scenario.id.clone()).collect();
    augmented_scenario_ids.sort();

    let augmented_feasible_ids: BTreeSet<String> = augmented.feasible_portfolios
        .iter().map(portfolio_key).collect();
    let mut original_feasible_removed_by_addition: Vec<Vec<String>> = original
        .feasible_portfolios.iter()
        .filter(|portfolio| !augmented_feasible_ids.contains(&portfolio_key(portfolio)))
        .map(|portfolio| portfolio.project_ids.clone())
        .collect();
    original_feasible_removed_by_addition.sort();

    let original_frontier_project_ids: Vec<Vec<String>> = original.pareto_frontier
        .iter().map(|portfolio| portfolio.project_ids.clone()).collect();
    let augmented_frontier_project_ids: Vec<Vec<String>> = augmented.pareto_frontier
        .iter().map(|portfolio| portfolio.project_ids.clone()).collect();
    let original_frontier_ids: BTreeSet<String> =
        original.pareto_frontier.iter().map(portfolio_key).collect();
    let augmented_frontier_ids: BTreeSet<String> =
        augmented.pareto_frontier.iter().map(portfolio_key).collect();
    let frontier_id_intersection_count = original_frontier_ids
        .intersection(&augmented_frontier_ids).count();
    let frontier_id_union_count = original_frontier_ids
        .union(&augmented_frontier_ids).count();

    if augmented.feasible_portfolios.len() > original.feasible_portfolios.len() {
        return Err("adding a scenario unexpectedly increased feasible portfolio count".to_owned());
    }

    Ok(ScenarioAdditionReport {
        added_scenario_id: patch.scenario.id.clone(),
        original_scenario_ids,
        augmented_scenario_ids,
        candidate_subset_count: augmented.candidate_subset_count,
        original_feasible_portfolio_count: original.feasible_portfolios.len(),
        augmented_feasible_portfolio_count: augmented.feasible_portfolios.len(),
        original_frontier_project_ids,
        augmented_frontier_project_ids,
        original_feasible_removed_by_addition,
        frontier_id_intersection_count,
        frontier_id_union_count,
    })
}

pub fn check_expected(evaluation: &Evaluation, expected_raw: &str) -> Result<(), String> {
    let expected: serde_json::Value =
        serde_json::from_str(expected_raw).map_err(|error| error.to_string())?;

    let expected_subsets = expected["candidate_subsets"]
        .as_u64()
        .ok_or_else(|| "expected candidate_subsets missing".to_owned())?;
    let expected_feasible_count = expected["feasible_count"]
        .as_u64()
        .ok_or_else(|| "expected feasible_count missing".to_owned())?;
    let expected_frontier_count = expected["pareto_frontier_count"]
        .as_u64()
        .ok_or_else(|| "expected pareto_frontier_count missing".to_owned())?;
    let expected_rejected_count = expected["rejected_subsets"]
        .as_u64()
        .ok_or_else(|| "expected rejected_subsets missing".to_owned())?;

    let expected_reason_counts: RejectionCounts =
        serde_json::from_value(expected["rejection_reason_counts"].clone())
            .map_err(|error| format!("expected rejection_reason_counts invalid: {error}"))?;
    if evaluation.candidate_subset_count != expected_subsets
        || evaluation.feasible_portfolios.len() as u64 != expected_feasible_count
        || evaluation.candidate_subset_count - evaluation.feasible_portfolios.len() as u64
            != expected_rejected_count
        || evaluation.pareto_frontier.len() as u64 != expected_frontier_count
        || evaluation.rejection_reason_counts != expected_reason_counts
        || evaluation.rejection_reason_counts.total() != expected_rejected_count
    {
        return Err("known-answer counts or rejection accounting mismatch".to_owned());
    }

    let expected_ids: Vec<Vec<String>> =
        serde_json::from_value(expected["feasible_portfolio_ids"].clone())
            .map_err(|error| error.to_string())?;
    let mut expected_ids: Vec<String> = expected_ids.iter().map(|ids| ids.join("+")).collect();
    let actual_ids: Vec<String> = evaluation
        .feasible_portfolios
        .iter()
        .map(portfolio_key)
        .collect();
    expected_ids.sort();
    if actual_ids != expected_ids {
        return Err("known-answer feasible set mismatch".to_owned());
    }

    let expected_frontier: Vec<Portfolio> =
        serde_json::from_value(expected["pareto_frontier"].clone())
            .map_err(|error| error.to_string())?;
    let mut expected_frontier = expected_frontier;
    expected_frontier.sort_by_key(portfolio_key);
    if evaluation.pareto_frontier != expected_frontier {
        return Err("known-answer frontier or objective/resource totals mismatch".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str =
        include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.json");
    const EXPECTED: &str =
        include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.expected.json");
    const V02_INPUT: &str =
        include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json");
    const V02_EXPECTED: &str = include_str!(
        "../../../docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.expected.json"
    );
    const V24_INPUT: &str =
        include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.json");
    const V24_SEARCH_EXPECTED: &str = include_str!(
        "../../../docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.search-expected.json"
    );
    const WITHHELD_SCENARIO: &str = include_str!(
        "../../../docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-withheld-compound-stress.json"
    );
    const NO_EFFECT_SCENARIO: &str = include_str!(
        "../../../docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-no-effect-control.json"
    );

    fn fixture() -> Input {
        serde_json::from_str(INPUT).expect("synthetic portfolio fixture should parse")
    }

    fn generated_v02_problem(project_count: usize) -> Input {
        let scenario_ids = ["normal", "hot_dry", "grid_stress"];
        let scenarios = scenario_ids
            .iter()
            .map(|id| Scenario {
                id: (*id).to_owned(),
                weight: None,
                probability: None,
                notes: "Deterministically generated synthetic stress case; no probability."
                    .to_owned(),
            })
            .collect();
        let service_baseline_by_scenario = scenario_ids
            .iter()
            .map(|id| ((*id).to_owned(), 1_i64))
            .collect();
        let objective_schema = OBJECTIVE_IDS
            .iter()
            .map(|id| ObjectiveDefinition {
                id: (*id).to_owned(),
                direction: "maximize".to_owned(),
                unit: "synthetic unit".to_owned(),
            })
            .collect();

        let projects = (0..project_count)
            .map(|index| {
                let id = format!("G{index:02}");
                let period = format!("P{}", (index % 3) + 1);
                let outcomes = scenario_ids
                    .iter()
                    .enumerate()
                    .map(|(scenario_index, scenario_id)| {
                        let i = index as i64;
                        let s = scenario_index as i64;
                        (
                            (*scenario_id).to_owned(),
                            ObjectiveVector {
                                economic_value_u: (i * 7 + s * 3) % 13 - 3,
                                essential_service_points: (i + s * 2) % 3,
                                paid_work_opportunity_points: (i * 5 + s * 2) % 9,
                                ecological_quality_points: (i * 2 + s * 5) % 11,
                            },
                        )
                    })
                    .collect();
                let gate = if index % 9 == 8 {
                    "UNKNOWN"
                } else if index % 13 == 12 {
                    "BLOCK"
                } else {
                    "PASS"
                };
                Project {
                    id,
                    name: format!("Generated synthetic option {index:02}"),
                    period,
                    stage_order: index as u32 + 1,
                    cost_u: 1 + (index % 3) as i64,
                    crew_u: 1 + (index % 2) as i64,
                    requires: Vec::new(),
                    gate: gate.to_owned(),
                    outcomes,
                    evidence_class: if gate == "PASS" {
                        "synthetic"
                    } else {
                        "synthetic hostile negative control"
                    }
                    .to_owned(),
                }
            })
            .collect();

        let budgets_by_period = (1..=3)
            .map(|period| PeriodBudget {
                period: format!("P{period}"),
                cost_cap_u: 4,
                crew_cap_u: 3,
            })
            .collect();

        let mut interaction_rules = Vec::new();
        if project_count >= 5 {
            interaction_rules.push(InteractionRule {
                id: "generated-synergy-00-04".to_owned(),
                kind: "synergy".to_owned(),
                when_all_selected: vec!["G00".to_owned(), "G04".to_owned()],
                objective_adjustment_by_scenario: scenario_ids
                    .iter()
                    .enumerate()
                    .map(|(index, scenario_id)| {
                        (
                            (*scenario_id).to_owned(),
                            ObjectiveVector {
                                economic_value_u: 2,
                                essential_service_points: 1,
                                paid_work_opportunity_points: 1,
                                ecological_quality_points: index as i64,
                            },
                        )
                    })
                    .collect(),
                reason: "Deterministic generated synthetic complementarity.".to_owned(),
            });
        }
        if project_count >= 8 {
            interaction_rules.push(InteractionRule {
                id: "generated-overlap-02-07".to_owned(),
                kind: "overlap_adjustment".to_owned(),
                when_all_selected: vec!["G02".to_owned(), "G07".to_owned()],
                objective_adjustment_by_scenario: scenario_ids
                    .iter()
                    .enumerate()
                    .map(|(index, scenario_id)| {
                        (
                            (*scenario_id).to_owned(),
                            ObjectiveVector {
                                economic_value_u: -1,
                                essential_service_points: 0,
                                paid_work_opportunity_points: -1,
                                ecological_quality_points: index as i64,
                            },
                        )
                    })
                    .collect(),
                reason: "Deterministic generated synthetic overlap adjustment.".to_owned(),
            });
        }

        Input {
            schema_version: "sa-economic-portfolio-v0.2".to_owned(),
            evidence_class: "synthetic".to_owned(),
            synthetic_only: true,
            claim_ceiling: "Deterministic generated synthetic search benchmark only.".to_owned(),
            budgets_by_period,
            scenarios,
            service_baseline_by_scenario,
            mandatory_floors: vec![Floor {
                id: "generated-service-floor".to_owned(),
                metric: "total_essential_service_points".to_owned(),
                direction: "minimize violation".to_owned(),
                threshold: 2,
                apply_to_each_scenario: true,
                source_class: "synthetic test invariant".to_owned(),
            }],
            objective_schema,
            projects,
            interaction_rules,
            enumeration: EnumerationConfig {
                method: "complete subset enumeration".to_owned(),
                candidate_subset_count: 1_u64 << project_count,
                over_cap_behavior:
                    "structured error; never return a partial frontier as complete".to_owned(),
                objective_comparison:
                    "scenario-wise componentwise maximization over every declared scenario and objective; scenario cases have no probabilities".to_owned(),
                canonicalization:
                    "sort project IDs within each portfolio and sort portfolio ID tuples lexicographically".to_owned(),
            },
        }
    }

    fn worst_case_objectives(portfolio: &Portfolio, scenarios: &[Scenario]) -> [i64; 4] {
        let mut values = [0_i64; 4];
        for (index, objective_id) in OBJECTIVE_IDS.iter().enumerate() {
            values[index] = scenarios
                .iter()
                .map(|scenario| {
                    portfolio.outcomes[&scenario.id]
                        .value(objective_id)
                        .expect("declared objective should exist")
                })
                .min()
                .expect("at least one scenario should exist");
        }
        values
    }

    fn normalized_weight_winner(
        evaluation: &Evaluation,
        scenarios: &[Scenario],
        weights: [f64; 4],
    ) -> String {
        let entries: Vec<(String, [i64; 4])> = evaluation
            .feasible_portfolios
            .iter()
            .map(|portfolio| {
                (
                    portfolio_key(portfolio),
                    worst_case_objectives(portfolio, scenarios),
                )
            })
            .collect();
        let mut minimum = [i64::MAX; 4];
        let mut maximum = [i64::MIN; 4];
        for (_, values) in &entries {
            for (index, value) in values.iter().enumerate() {
                minimum[index] = minimum[index].min(*value);
                maximum[index] = maximum[index].max(*value);
            }
        }

        let mut scored: Vec<(String, f64)> = entries
            .into_iter()
            .map(|(key, values)| {
                let score = weights
                    .iter()
                    .enumerate()
                    .map(|(index, weight)| {
                        let normalized = if maximum[index] == minimum[index] {
                            1.0
                        } else {
                            (values[index] - minimum[index]) as f64
                                / (maximum[index] - minimum[index]) as f64
                        };
                        weight * normalized
                    })
                    .sum();
                (key, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        scored
            .first()
            .expect("a feasible portfolio should exist")
            .0
            .clone()
    }

    fn project_robustly_dominates(
        candidate: &Project,
        other: &Project,
        scenarios: &[Scenario],
    ) -> bool {
        let mut strictly_better = false;
        for scenario in scenarios {
            for objective_id in OBJECTIVE_IDS {
                let candidate_value = candidate.outcomes[&scenario.id]
                    .value(objective_id)
                    .expect("declared objective should exist");
                let other_value = other.outcomes[&scenario.id]
                    .value(objective_id)
                    .expect("declared objective should exist");
                if candidate_value < other_value {
                    return false;
                }
                if candidate_value > other_value {
                    strictly_better = true;
                }
            }
        }
        strictly_better
    }

    #[test]
    fn deterministic_generated_instances_match_exact_oracle_at_multiple_sizes() {
        for project_count in [12_usize, 16, 18] {
            let input = generated_v02_problem(project_count);
            let exact = evaluate(&input).expect("generated exact oracle should evaluate");
            let searched = evaluate_frontier_search(&input)
                .expect("generated frontier search should evaluate");
            assert_eq!(
                searched.pareto_frontier, exact.pareto_frontier,
                "frontier mismatch for {project_count} generated projects"
            );
            assert_eq!(
                searched.stats.feasible_portfolio_count,
                exact.feasible_portfolios.len() as u64
            );
            assert_eq!(
                searched.stats.accounted_subset_count(),
                1_u64 << project_count
            );
            assert!(searched.stats.visited_nodes < (1_u64 << (project_count + 1)) - 1);
        }
    }
    #[test]
    fn generalized_v02_fixture_matches_exact_oracle_and_search_frontier() {
        let input: Input =
            serde_json::from_str(V02_INPUT).expect("v0.2 synthetic fixture should parse");
        let exact = evaluate(&input).expect("ten-project exact oracle should evaluate");
        check_expected(&exact, V02_EXPECTED)
            .expect("v0.2 fixture should match its independent known-answer manifest");
        assert_eq!(exact.candidate_subset_count, 1_024);
        assert_eq!(exact.feasible_portfolios.len(), 82);
        assert_eq!(exact.pareto_frontier.len(), 8);

        let searched =
            evaluate_frontier_search(&input).expect("ten-project frontier search should evaluate");
        assert_eq!(searched.pareto_frontier, exact.pareto_frontier);
        assert_eq!(
            searched.stats.feasible_portfolio_count,
            exact.feasible_portfolios.len() as u64
        );
        assert_eq!(searched.stats.accounted_subset_count(), 1_024);
        assert!(searched.stats.visited_nodes > 0);
        assert!(searched.stats.visited_nodes < 2_047);
        assert!(searched.stats.gate_pruned_subsets > 0);
        assert!(searched.stats.budget_pruned_subsets > 0);
    }

    #[test]
    fn generalized_v02_row_permutations_preserve_exact_and_search_results() {
        let mut input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture should parse");
        let baseline = evaluate(&input).expect("baseline exact evaluation should work");
        let baseline_search =
            evaluate_frontier_search(&input).expect("baseline frontier search should work");
        input.projects.reverse();
        input.scenarios.reverse();
        input.budgets_by_period.reverse();
        input.objective_schema.reverse();
        input.interaction_rules.reverse();
        let permuted = evaluate(&input).expect("permuted exact evaluation should work");
        let permuted_search =
            evaluate_frontier_search(&input).expect("permuted frontier search should work");
        assert_eq!(permuted, baseline);
        assert_eq!(
            permuted_search.pareto_frontier,
            baseline_search.pareto_frontier
        );
        assert_eq!(
            permuted_search.stats.feasible_portfolio_count,
            baseline_search.stats.feasible_portfolio_count
        );
        assert_eq!(permuted_search.stats.accounted_subset_count(), 1_024);
    }

    #[test]
    fn dependency_aware_search_matches_v01_exact_frontier() {
        let input = fixture();
        let exact = evaluate(&input).expect("v0.1 exact oracle should evaluate");
        let searched = evaluate_frontier_search(&input)
            .expect("v0.1 fixture should be accepted by the v0.2-compatible search");
        assert_eq!(searched.pareto_frontier, exact.pareto_frontier);
        assert_eq!(
            searched.stats.feasible_portfolio_count,
            exact.feasible_portfolios.len() as u64
        );
        assert_eq!(searched.stats.accounted_subset_count(), 256);
    }

    #[test]
    fn frontier_search_prunes_a_24_project_hostile_case_without_partial_claims() {
        let input: Input =
            serde_json::from_str(V24_INPUT).expect("24-project hostile fixture should parse");
        let reference: serde_json::Value = serde_json::from_str(V24_SEARCH_EXPECTED)
            .expect("24-project search manifest should parse");
        let expected_frontier: Vec<Portfolio> =
            serde_json::from_value(reference["pareto_frontier"].clone())
                .expect("search manifest must carry the expected frontier");

        let searched = evaluate_frontier_search(&input)
            .expect("24-project search should prune unknown-gate branches");
        assert_eq!(searched.stats.candidate_subset_count, 1_u64 << 24);
        assert_eq!(searched.stats.accounted_subset_count(), 1_u64 << 24);
        assert!(searched.stats.gate_pruned_subsets > 0);
        assert_eq!(
            searched.pareto_frontier, expected_frontier,
            "unknown-gate decoys must never alter the known eight-project frontier"
        );
        assert_eq!(searched.pareto_frontier.len(), 4);
        assert!(
            searched
                .pareto_frontier
                .iter()
                .all(|portfolio| { portfolio.project_ids.iter().all(|id| !id.starts_with('N')) })
        );
        assert!(searched.stats.visited_nodes < 1_000_000);
        assert_eq!(
            searched.stats.feasible_portfolio_count,
            reference["feasible_portfolio_count"].as_u64().unwrap()
        );
    }

    #[test]
    fn frozen_fixture_matches_exact_feasible_set_and_frontier() {
        let input = fixture();
        let evaluation = evaluate(&input).expect("fixture should evaluate");
        check_expected(&evaluation, EXPECTED).expect("fixture must match known-answer oracle");
        assert_eq!(evaluation.candidate_subset_count, 256);
        assert_eq!(evaluation.feasible_portfolios.len(), 22);
        assert_eq!(evaluation.pareto_frontier.len(), 4);
    }

    #[test]
    fn row_permutations_do_not_change_canonical_result() {
        let input = fixture();
        let baseline = evaluate(&input).expect("fixture should evaluate");
        let mut permuted = fixture();
        permuted.projects.reverse();
        permuted.scenarios.reverse();
        permuted.budgets_by_period.reverse();
        permuted.objective_schema.reverse();
        permuted.interaction_rules.reverse();
        let after = evaluate(&permuted).expect("permuted fixture should evaluate");
        assert_eq!(baseline, after);
    }

    #[test]
    fn unknown_gate_decoy_never_enters_feasible_set_or_frontier() {
        let evaluation = evaluate(&fixture()).expect("fixture should evaluate");
        assert!(evaluation.feasible_portfolios.iter().all(|portfolio| {
            !portfolio
                .project_ids
                .iter()
                .any(|project_id| project_id == "X")
        }));
        assert!(evaluation.pareto_frontier.iter().all(|portfolio| {
            !portfolio
                .project_ids
                .iter()
                .any(|project_id| project_id == "X")
        }));
    }

    #[test]
    fn unknown_prerequisite_fails_closed() {
        let mut input = fixture();
        let water = input
            .projects
            .iter_mut()
            .find(|project| project.id == "W")
            .unwrap();
        water.requires = vec!["MISSING".to_owned()];
        assert!(evaluate(&input).is_err());
    }

    #[test]
    fn over_cap_corpus_is_rejected_instead_of_truncated() {
        let mut input = fixture();
        let extra = input.projects[0].clone();
        input.projects.extend(std::iter::repeat_n(extra, 13));
        assert!(evaluate(&input).is_err());
    }

    #[test]
    fn rejection_reasons_reconcile_every_subset_with_frozen_precedence() {
        let evaluation = evaluate(&fixture()).expect("fixture should evaluate");
        assert_eq!(evaluation.candidate_subset_count, 256);
        assert_eq!(
            evaluation.feasible_portfolios.len() as u64
                + evaluation.rejection_reason_counts.total(),
            256
        );
        assert_eq!(evaluation.rejection_reason_counts.gate_not_pass, 128);
        assert_eq!(evaluation.rejection_reason_counts.missing_prerequisite, 48);
        assert_eq!(
            evaluation.rejection_reason_counts.period_budget_exceeded,
            40
        );
        assert_eq!(evaluation.rejection_reason_counts.crew_capacity_exceeded, 0);
        assert_eq!(
            evaluation.rejection_reason_counts.protected_service_floor,
            18
        );
    }

    #[test]
    fn crew_capacity_has_separate_rejection_accounting_when_budget_is_not_binding() {
        let mut input = fixture();
        for budget in &mut input.budgets_by_period {
            budget.cost_cap_u = 100;
            budget.crew_cap_u = 1;
        }
        let evaluation = evaluate(&input).expect("modified synthetic case should evaluate");
        assert!(evaluation.rejection_reason_counts.crew_capacity_exceeded > 0);
        assert_eq!(
            evaluation.feasible_portfolios.len() as u64
                + evaluation.rejection_reason_counts.total(),
            evaluation.candidate_subset_count
        );
        for portfolio in &evaluation.feasible_portfolios {
            assert!(
                portfolio
                    .period_cost_and_crew
                    .values()
                    .all(|used| used.crew_u <= 1)
            );
        }
    }

    #[test]
    fn stricter_floor_never_returns_a_below_floor_portfolio() {
        let mut input = fixture();
        for floor in &mut input.mandatory_floors {
            floor.threshold = 99;
        }
        let evaluation = evaluate(&input).expect("strict synthetic floor should evaluate");
        assert!(evaluation.feasible_portfolios.is_empty());
        assert!(evaluation.pareto_frontier.is_empty());
        assert!(evaluation.rejection_reason_counts.protected_service_floor > 0);
        assert_eq!(
            evaluation.rejection_reason_counts.total(),
            evaluation.candidate_subset_count
        );
    }

    #[test]
    fn tighter_period_budget_is_reported_and_never_exceeded() {
        let mut input = fixture();
        input
            .budgets_by_period
            .iter_mut()
            .find(|b| b.period == "P1")
            .unwrap()
            .cost_cap_u = 4;
        let evaluation = evaluate(&input).expect("tighter-budget synthetic case should evaluate");
        assert!(evaluation.rejection_reason_counts.period_budget_exceeded > 0);
        assert!(
            evaluation
                .feasible_portfolios
                .iter()
                .all(|portfolio| { portfolio.period_cost_and_crew["P1"].cost_u <= 4 })
        );
    }

    #[test]
    fn corrupted_known_answer_manifest_is_rejected() {
        let evaluation = evaluate(&fixture()).expect("fixture should evaluate");
        let corrupted = EXPECTED.replace("\"feasible_count\": 22", "\"feasible_count\": 23");
        assert_ne!(corrupted, EXPECTED);
        assert!(check_expected(&evaluation, &corrupted).is_err());
    }

    #[test]
    fn minimum_cost_baseline_returns_the_expected_feasible_portfolio() {
        let evaluation = evaluate(&fixture()).expect("fixture should evaluate");
        let winner = evaluation
            .feasible_portfolios
            .iter()
            .min_by_key(|portfolio| {
                let cost = portfolio
                    .period_cost_and_crew
                    .values()
                    .map(|used| used.cost_u)
                    .sum::<i64>();
                let crew = portfolio
                    .period_cost_and_crew
                    .values()
                    .map(|used| used.crew_u)
                    .sum::<i64>();
                (cost, crew, portfolio_key(portfolio))
            })
            .expect("a feasible portfolio should exist");
        assert_eq!(portfolio_key(winner), "A+J");
        assert_eq!(
            winner
                .period_cost_and_crew
                .values()
                .map(|used| used.cost_u)
                .sum::<i64>(),
            4
        );
    }

    #[test]
    fn normalized_weight_baseline_exposes_preference_sensitive_selection() {
        let input = fixture();
        let evaluation = evaluate(&input).expect("fixture should evaluate");
        let scenarios = input.scenarios.as_slice();

        assert_eq!(
            normalized_weight_winner(&evaluation, scenarios, [0.25, 0.25, 0.25, 0.25]),
            "I+J+T+W"
        );
        assert_eq!(
            normalized_weight_winner(&evaluation, scenarios, [0.55, 0.20, 0.15, 0.10]),
            "I+J+T+W"
        );
        assert_eq!(
            normalized_weight_winner(&evaluation, scenarios, [0.10, 0.60, 0.20, 0.10]),
            "I+J+T+W"
        );
        assert_eq!(
            normalized_weight_winner(&evaluation, scenarios, [0.10, 0.15, 0.65, 0.10]),
            "I+J+R+T"
        );
        assert_eq!(
            normalized_weight_winner(&evaluation, scenarios, [0.10, 0.10, 0.10, 0.70]),
            "F+J+T+W"
        );
    }

    #[test]
    fn naive_individual_project_pareto_pruning_loses_the_global_frontier() {
        let input = fixture();
        let evaluation = evaluate(&input).expect("fixture should evaluate");
        let pass_projects: Vec<&Project> = input
            .projects
            .iter()
            .filter(|project| project.gate == "PASS")
            .collect();
        let dominated_project_ids: BTreeSet<String> = pass_projects
            .iter()
            .filter(|project| {
                pass_projects.iter().any(|candidate| {
                    candidate.id != project.id
                        && project_robustly_dominates(candidate, project, &input.scenarios)
                })
            })
            .map(|project| project.id.clone())
            .collect();
        assert_eq!(
            dominated_project_ids,
            BTreeSet::from(["A".to_owned(), "T".to_owned()])
        );

        let retained_project_ids: BTreeSet<String> = pass_projects
            .iter()
            .filter(|project| !dominated_project_ids.contains(&project.id))
            .map(|project| project.id.clone())
            .collect();
        let mut surviving: Vec<&Portfolio> = evaluation
            .feasible_portfolios
            .iter()
            .filter(|portfolio| {
                portfolio
                    .project_ids
                    .iter()
                    .all(|project_id| retained_project_ids.contains(project_id))
            })
            .collect();
        surviving.sort_by_key(|portfolio| portfolio_key(portfolio));
        assert_eq!(
            surviving
                .iter()
                .map(|portfolio| portfolio_key(portfolio))
                .collect::<Vec<_>>(),
            vec!["F+J".to_owned(), "F+J+R".to_owned(), "J+R".to_owned()]
        );

        let mut surviving_frontier: Vec<&Portfolio> = Vec::new();
        for (candidate_index, candidate) in surviving.iter().enumerate() {
            let is_dominated = surviving.iter().enumerate().any(|(other_index, other)| {
                other_index != candidate_index && dominates(other, candidate, &input.scenarios)
            });
            if !is_dominated {
                surviving_frontier.push(*candidate);
            }
        }
        assert_eq!(
            surviving_frontier
                .iter()
                .map(|portfolio| portfolio_key(portfolio))
                .collect::<Vec<_>>(),
            vec!["F+J+R".to_owned()]
        );
        assert!(evaluation.pareto_frontier.iter().all(|portfolio| {
            portfolio
                .project_ids
                .iter()
                .any(|project_id| dominated_project_ids.contains(project_id))
        }));
    }

    #[test]
    fn scenario_omission_rescores_frontier_on_the_full_reference_universe() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture should parse");
        let omitted_id = input.scenarios[0].id.clone();
        let report = evaluate_scenario_omission(&input, &omitted_id)
            .expect("valid scenario omission should be evaluable");

        assert_eq!(report.omitted_scenario_id, omitted_id);
        assert_eq!(report.original_scenario_ids.len(), input.scenarios.len());
        assert_eq!(report.reduced_scenario_ids.len() + 1, input.scenarios.len());
        assert_eq!(
            report.candidate_subset_count,
            input.enumeration.candidate_subset_count
        );
        assert_eq!(
            report.reference_frontier_project_ids.len(),
            evaluate(&input).unwrap().pareto_frontier.len()
        );
        assert!(report.frontier_id_intersection_count <= report.frontier_id_union_count);
        assert_eq!(
            report.full_reference_infeasible_frontier_count,
            report
                .reduced_frontier
                .iter()
                .filter(|portfolio| !portfolio.full_reference_feasible)
                .count()
        );

        for portfolio in &report.reduced_frontier {
            if !portfolio.full_reference_feasible {
                assert!(portfolio.reference_outcomes_by_scenario.is_none());
                assert!(portfolio.regret_by_scenario.is_none());
                assert!(portfolio.worst_case_regret.is_none());
                continue;
            }

            let outcomes = portfolio
                .reference_outcomes_by_scenario
                .as_ref()
                .expect("feasible portfolio has full reference outcomes");
            let regrets = portfolio
                .regret_by_scenario
                .as_ref()
                .expect("feasible portfolio has objective-wise regrets");
            let worst = portfolio
                .worst_case_regret
                .as_ref()
                .expect("feasible portfolio has worst-case regret");
            assert_eq!(outcomes.len(), input.scenarios.len());
            assert_eq!(regrets.len(), input.scenarios.len());
            for regret in regrets.values() {
                assert!(regret.economic_value_u >= 0);
                assert!(regret.essential_service_points >= 0);
                assert!(regret.paid_work_opportunity_points >= 0);
                assert!(regret.ecological_quality_points >= 0);
                assert!(worst.economic_value_u >= regret.economic_value_u);
                assert!(worst.essential_service_points >= regret.essential_service_points);
                assert!(worst.paid_work_opportunity_points >= regret.paid_work_opportunity_points);
                assert!(worst.ecological_quality_points >= regret.ecological_quality_points);
            }
        }
    }

    #[test]
    fn scenario_omission_is_deterministic_and_fails_closed() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture should parse");
        let omitted_id = input.scenarios[0].id.clone();
        let first = evaluate_scenario_omission(&input, &omitted_id)
            .expect("valid scenario omission should be evaluable");
        let second = evaluate_scenario_omission(&input, &omitted_id)
            .expect("repeated scenario omission should be evaluable");
        assert_eq!(first, second);

        assert!(omit_scenario(&input, "absent-scenario").is_err());
        assert!(evaluate_scenario_omission(&input, "absent-scenario").is_err());

        let one_scenario =
            omit_scenario(&input, &omitted_id).expect("one scenario can be removed from two");
        let remaining_id = one_scenario.scenarios[0].id.clone();
        assert!(omit_scenario(&one_scenario, &remaining_id).is_err());

        assert!(omit_scenario(&fixture(), "nominal").is_err());
    }

    #[test]
    fn scenario_omission_is_invariant_to_scenario_row_order() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture should parse");
        let omitted_id = input.scenarios[0].id.clone();
        let original = evaluate_scenario_omission(&input, &omitted_id)
            .expect("valid scenario omission should be evaluable");
        let mut permuted = input.clone();
        permuted.scenarios.reverse();
        let reordered = evaluate_scenario_omission(&permuted, &omitted_id)
            .expect("permuted scenario omission should be evaluable");
        assert_eq!(original, reordered);
    }

    #[test]
    fn scenario_addition_is_fail_closed_and_preserves_input() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture parses");
        let mut patch: ScenarioPatch = serde_json::from_str(WITHHELD_SCENARIO)
            .expect("withheld scenario patch parses");
        let original = input.clone();
        let augmented = add_scenario(&input, &patch).expect("valid scenario is addable");
        assert_eq!(input, original);
        assert_eq!(augmented.scenarios.len(), input.scenarios.len() + 1);
        assert_eq!(augmented.scenarios.last().unwrap().id, patch.scenario.id);

        patch.scenario.id = "normal_reference".to_owned();
        assert!(add_scenario(&input, &patch).is_err());
        patch.scenario.id = "compound_heat_drought_grid_crew".to_owned();
        patch.project_outcomes_by_id.remove("P00");
        assert!(add_scenario(&input, &patch).is_err());
    }

    #[test]
    fn scenario_addition_cannot_increase_feasibility_and_reports_lost_portfolios() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture parses");
        let patch: ScenarioPatch = serde_json::from_str(WITHHELD_SCENARIO)
            .expect("withheld scenario patch parses");
        let original = evaluate(&input).expect("original exact evaluation");
        let augmented_input = add_scenario(&input, &patch).expect("valid addition");
        let augmented = evaluate(&augmented_input).expect("augmented exact evaluation");
        let report = evaluate_scenario_addition(&input, &patch)
            .expect("addition report should evaluate");

        let original_ids: BTreeSet<String> = original.feasible_portfolios
            .iter().map(portfolio_key).collect();
        let augmented_ids: BTreeSet<String> = augmented.feasible_portfolios
            .iter().map(portfolio_key).collect();
        assert!(augmented_ids.is_subset(&original_ids));
        assert_eq!(
            report.original_feasible_removed_by_addition.len(),
            original_ids.difference(&augmented_ids).count()
        );
        assert_eq!(report.original_feasible_portfolio_count, original.feasible_portfolios.len());
        assert_eq!(report.augmented_feasible_portfolio_count, augmented.feasible_portfolios.len());
        assert_eq!(
            report.augmented_frontier_project_ids,
            augmented.pareto_frontier.iter()
                .map(|portfolio| portfolio.project_ids.clone()).collect::<Vec<_>>()
        );
        assert!(report.frontier_id_intersection_count <= report.frontier_id_union_count);
    }

    #[test]
    fn scenario_addition_report_is_deterministic_and_row_order_invariant() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture parses");
        let patch: ScenarioPatch = serde_json::from_str(WITHHELD_SCENARIO)
            .expect("withheld scenario patch parses");
        let first = evaluate_scenario_addition(&input, &patch).unwrap();
        assert_eq!(first, evaluate_scenario_addition(&input, &patch).unwrap());

        let mut permuted_input = input.clone();
        permuted_input.scenarios.reverse();
        let mut permuted_patch = patch.clone();
        permuted_patch.project_outcomes_by_id =
            patch.project_outcomes_by_id.iter().rev()
                .map(|(id, outcome)| (id.clone(), outcome.clone())).collect();
        assert_eq!(
            first,
            evaluate_scenario_addition(&permuted_input, &permuted_patch).unwrap()
        );
    }

    #[test]
    fn no_effect_scenario_addition_preserves_feasible_set_and_frontier() {
        let input: Input = serde_json::from_str(V02_INPUT).expect("v0.2 fixture parses");
        let patch: ScenarioPatch =
            serde_json::from_str(NO_EFFECT_SCENARIO).expect("no-effect control patch parses");
        let report = evaluate_scenario_addition(&input, &patch)
            .expect("duplicate scenario should be addable");

        assert_eq!(report.original_feasible_portfolio_count, 82);
        assert_eq!(report.augmented_feasible_portfolio_count, 82);
        assert!(report.original_feasible_removed_by_addition.is_empty());
        assert_eq!(
            report.frontier_id_intersection_count,
            report.frontier_id_union_count
        );
        assert_eq!(
            report.original_frontier_project_ids,
            report.augmented_frontier_project_ids
        );
    }

    #[test]
    fn interaction_adjustments_are_applied_once_per_selected_pair() {
        let evaluation = evaluate(&fixture()).expect("fixture should evaluate");
        let portfolio = evaluation
            .pareto_frontier
            .iter()
            .find(|portfolio| portfolio_key(portfolio) == "A+F+J+R")
            .expect("A+F+J+R should remain on the frontier");
        assert_eq!(portfolio.outcomes["nominal"].ecological_quality_points, 13);
        assert_eq!(
            portfolio.outcomes["hot_dry_grid_stress"].ecological_quality_points,
            13
        );

        let water_portfolio = evaluation
            .pareto_frontier
            .iter()
            .find(|portfolio| portfolio_key(portfolio) == "F+J+T+W")
            .expect("F+J+T+W should remain on the frontier");
        assert_eq!(
            water_portfolio.outcomes["nominal"].essential_service_points,
            12
        );
        assert_eq!(
            water_portfolio.outcomes["hot_dry_grid_stress"].essential_service_points,
            11
        );
    }
}
