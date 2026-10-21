use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-ACTIVE-EVIDENCE-001";
const MAX_HYPOTHESES: usize = 20;
const MAX_QUERIES: usize = 16;
const MAX_STATES: usize = 50_000;
const MAX_POLICY_COMBINATIONS: usize = 100_000;
const MAX_FRONTIER_PER_STATE: usize = 128;
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_active_evidence_scenarios_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_active_evidence_v0_1.schema.json";

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Cost {
    staff_minutes: u32,
    cash_zar_cents: u32,
    delay_hours: u32,
}

impl Cost {
    fn checked_add(self, other: Self) -> Result<Self, String> {
        Ok(Self {
            staff_minutes: self
                .staff_minutes
                .checked_add(other.staff_minutes)
                .ok_or_else(|| "staff-minute cost overflow".to_string())?,
            cash_zar_cents: self
                .cash_zar_cents
                .checked_add(other.cash_zar_cents)
                .ok_or_else(|| "cash cost overflow".to_string())?,
            delay_hours: self
                .delay_hours
                .checked_add(other.delay_hours)
                .ok_or_else(|| "delay cost overflow".to_string())?,
        })
    }

    fn component_max(self, other: Self) -> Self {
        Self {
            staff_minutes: self.staff_minutes.max(other.staff_minutes),
            cash_zar_cents: self.cash_zar_cents.max(other.cash_zar_cents),
            delay_hours: self.delay_hours.max(other.delay_hours),
        }
    }

    fn fits(self, budget: Self) -> bool {
        self.staff_minutes <= budget.staff_minutes
            && self.cash_zar_cents <= budget.cash_zar_cents
            && self.delay_hours <= budget.delay_hours
    }

    fn weakly_dominates(self, other: Self) -> bool {
        self.staff_minutes <= other.staff_minutes
            && self.cash_zar_cents <= other.cash_zar_cents
            && self.delay_hours <= other.delay_hours
    }

    fn strictly_dominates(self, other: Self) -> bool {
        self.weakly_dominates(other) && self != other
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum ResolutionClass {
    Unresolved,
    Partial,
    Conditional,
    Guaranteed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct Policy {
    resolution_class: ResolutionClass,
    worst_case_cost: Cost,
    tree: Value,
}

#[derive(Debug, Clone, Deserialize)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize)]
struct Scenario {
    id: String,
    description: String,
    hypotheses: Vec<Hypothesis>,
    queries: Vec<Query>,
    budget: Cost,
    expected_status: String,
    expected_remaining_hypotheses: Vec<String>,
    #[serde(default)]
    hand_reference_frontier: Vec<ReferenceFrontier>,
}

#[derive(Debug, Clone, Deserialize)]
struct Hypothesis {
    id: String,
    description: String,
    decision_class: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Query {
    id: String,
    description: String,
    permitted: bool,
    access_basis: String,
    #[serde(default)]
    denial_reason: Option<String>,
    cost: Cost,
    outcomes: Vec<Outcome>,
}

#[derive(Debug, Clone, Deserialize)]
struct Outcome {
    outcome_id: String,
    kind: String,
    compatible_hypotheses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReferenceFrontier {
    reference_id: String,
    cost: Cost,
    policy_outline: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq)]
struct ScenarioReceipt {
    scenario_id: String,
    hypothesis_count: usize,
    budget: Cost,
    permitted_query_count: usize,
    states_explored: usize,
    policy_combinations_explored: usize,
    oracle_limit: String,
    frontier: Vec<Policy>,
    expected_contract: String,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    schema_sha256: String,
    scenario_count: usize,
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
        Err(format!("JSON Schema validation failed:\n{}", errors.join("\n")))
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
        errors.push("this verifier accepts only synthetic contract fixtures".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES ARE SYNTHETIC")
    {
        errors.push("fixture_notice must visibly identify all values as synthetic".to_string());
    }
    if corpus.claim_ceiling.trim().is_empty() {
        errors.push("claim_ceiling cannot be empty".to_string());
    }

    if corpus.scenarios.is_empty() {
        errors.push("scenario corpus cannot be empty".to_string());
    }
    let mut scenario_ids = BTreeSet::new();
    for scenario in &corpus.scenarios {
        if !scenario_ids.insert(scenario.id.clone()) {
            errors.push(format!("{}: duplicate scenario ID", scenario.id));
        }
        if scenario.hypotheses.len() < 2 || scenario.hypotheses.len() > MAX_HYPOTHESES {
            errors.push(format!(
                "{}: hypothesis count must be between 2 and {MAX_HYPOTHESES}",
                scenario.id
            ));
        }
        if scenario.queries.is_empty() || scenario.queries.len() > MAX_QUERIES {
            errors.push(format!(
                "{}: query count must be between 1 and {MAX_QUERIES}",
                scenario.id
            ));
        }

        let mut hypothesis_ids = BTreeSet::new();
        for hypothesis in &scenario.hypotheses {
            if !hypothesis_ids.insert(hypothesis.id.clone()) {
                errors.push(format!(
                    "{}: duplicate hypothesis ID {}",
                    scenario.id, hypothesis.id
                ));
            }
            if hypothesis.decision_class.trim().is_empty() {
                errors.push(format!(
                    "{} / {}: decision_class cannot be empty",
                    scenario.id, hypothesis.id
                ));
            }
        }

        let mut query_ids = BTreeSet::new();
        for query in &scenario.queries {
            if !query_ids.insert(query.id.clone()) {
                errors.push(format!(
                    "{}: duplicate query ID {}",
                    scenario.id, query.id
                ));
            }
            let access_forbidden = matches!(
                query.access_basis.as_str(),
                "restricted_unavailable" | "prohibited_sensitive_inference"
            );
            if query.permitted && access_forbidden {
                errors.push(format!(
                    "{} / {}: query cannot be permitted under access basis {}",
                    scenario.id, query.id, query.access_basis
                ));
            }
            if query.permitted && query.denial_reason.is_some() {
                errors.push(format!(
                    "{} / {}: permitted query cannot have a denial_reason",
                    scenario.id, query.id
                ));
            }
            if !query.permitted && query.denial_reason.as_deref().unwrap_or("").trim().is_empty()
            {
                errors.push(format!(
                    "{} / {}: forbidden query needs a denial_reason",
                    scenario.id, query.id
                ));
            }

            let mut outcome_ids = BTreeSet::new();
            let mut known_counts: BTreeMap<&str, usize> = BTreeMap::new();
            let mut unknown_count = 0usize;
            for outcome in &query.outcomes {
                if !outcome_ids.insert(outcome.outcome_id.clone()) {
                    errors.push(format!(
                        "{} / {}: duplicate outcome ID {}",
                        scenario.id, query.id, outcome.outcome_id
                    ));
                }
                let mut local_ids = BTreeSet::new();
                for hypothesis_id in &outcome.compatible_hypotheses {
                    if !hypothesis_ids.contains(hypothesis_id) {
                        errors.push(format!(
                            "{} / {} / {}: unknown hypothesis {}",
                            scenario.id, query.id, outcome.outcome_id, hypothesis_id
                        ));
                    }
                    if !local_ids.insert(hypothesis_id.as_str()) {
                        errors.push(format!(
                            "{} / {} / {}: duplicate compatible hypothesis {}",
                            scenario.id, query.id, outcome.outcome_id, hypothesis_id
                        ));
                    }
                }
                match outcome.kind.as_str() {
                    "observed" => {
                        for hypothesis_id in &outcome.compatible_hypotheses {
                            *known_counts.entry(hypothesis_id.as_str()).or_default() += 1;
                        }
                    }
                    "unknown_or_unmodelled" => {
                        unknown_count += 1;
                        let compatible: BTreeSet<&str> = outcome
                            .compatible_hypotheses
                            .iter()
                            .map(String::as_str)
                            .collect();
                        let all: BTreeSet<&str> =
                            hypothesis_ids.iter().map(String::as_str).collect();
                        if compatible != all {
                            errors.push(format!(
                                "{} / {} / {}: unknown outcome must preserve every hypothesis",
                                scenario.id, query.id, outcome.outcome_id
                            ));
                        }
                    }
                    other => errors.push(format!(
                        "{} / {} / {}: unsupported outcome kind {other}",
                        scenario.id, query.id, outcome.outcome_id
                    )),
                }
            }
            if unknown_count > 1 {
                errors.push(format!(
                    "{} / {}: at most one unknown_or_unmodelled outcome is supported",
                    scenario.id, query.id
                ));
            }
            for hypothesis_id in &hypothesis_ids {
                if known_counts.get(hypothesis_id.as_str()).copied().unwrap_or(0) != 1 {
                    errors.push(format!(
                        "{} / {}: observed outcomes must partition hypothesis {} exactly once",
                        scenario.id, query.id, hypothesis_id
                    ));
                }
            }
        }

        for hypothesis_id in &scenario.expected_remaining_hypotheses {
            if !hypothesis_ids.contains(hypothesis_id) {
                errors.push(format!(
                    "{}: expected_remaining_hypotheses references unknown ID {}",
                    scenario.id, hypothesis_id
                ));
            }
        }
        for reference in &scenario.hand_reference_frontier {
            if !reference.cost.fits(scenario.budget) {
                errors.push(format!(
                    "{} / {}: hand reference cost exceeds declared budget",
                    scenario.id, reference.reference_id
                ));
            }
            if reference.policy_outline.is_empty() {
                errors.push(format!(
                    "{} / {}: policy_outline cannot be empty",
                    scenario.id, reference.reference_id
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

fn canonicalize(corpus: &mut Corpus) {
    corpus.scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    for scenario in &mut corpus.scenarios {
        scenario.hypotheses.sort_by(|a, b| a.id.cmp(&b.id));
        scenario.queries.sort_by(|a, b| a.id.cmp(&b.id));
        scenario.expected_remaining_hypotheses.sort();
        scenario
            .hand_reference_frontier
            .sort_by(|a, b| a.reference_id.cmp(&b.reference_id));
        for query in &mut scenario.queries {
            query.outcomes.sort_by(|a, b| {
                (a.kind.as_str(), a.outcome_id.as_str())
                    .cmp(&(b.kind.as_str(), b.outcome_id.as_str()))
            });
            for outcome in &mut query.outcomes {
                outcome.compatible_hypotheses.sort();
            }
        }
    }
}

struct Solver<'a> {
    scenario: &'a Scenario,
    hypothesis_index: HashMap<String, usize>,
    budget: Cost,
    memo: BTreeMap<(u64, u64), Vec<Policy>>,
    states_explored: usize,
    policy_combinations_explored: usize,
}

impl<'a> Solver<'a> {
    fn new(scenario: &'a Scenario) -> Self {
        let hypothesis_index = scenario
            .hypotheses
            .iter()
            .enumerate()
            .map(|(index, hypothesis)| (hypothesis.id.clone(), index))
            .collect();
        Self {
            scenario,
            hypothesis_index,
            budget: scenario.budget,
            memo: BTreeMap::new(),
            states_explored: 0,
            policy_combinations_explored: 0,
        }
    }

    fn all_hypotheses_mask(&self) -> u64 {
        (1u64 << self.scenario.hypotheses.len()) - 1
    }

    fn all_queries_mask(&self) -> u64 {
        (1u64 << self.scenario.queries.len()) - 1
    }

    fn mask_for_ids(&self, ids: &[String]) -> Result<u64, String> {
        let mut mask = 0u64;
        for id in ids {
            let index = self
                .hypothesis_index
                .get(id)
                .ok_or_else(|| format!("{}: unknown hypothesis ID {id}", self.scenario.id))?;
            mask |= 1u64 << index;
        }
        Ok(mask)
    }

    fn hypothesis_ids(&self, mask: u64) -> Vec<String> {
        self.scenario
            .hypotheses
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1u64 << index) != 0)
            .map(|(_, hypothesis)| hypothesis.id.clone())
            .collect()
    }

    fn common_decision_class(&self, mask: u64) -> Option<String> {
        let classes: BTreeSet<&str> = self
            .scenario
            .hypotheses
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1u64 << index) != 0)
            .map(|(_, hypothesis)| hypothesis.decision_class.as_str())
            .collect();
        if classes.len() == 1 {
            classes.iter().next().map(|value| (*value).to_string())
        } else {
            None
        }
    }

    fn solve(&mut self, belief: u64, available_queries: u64) -> Result<Vec<Policy>, String> {
        if let Some(cached) = self.memo.get(&(belief, available_queries)) {
            return Ok(cached.clone());
        }
        self.states_explored += 1;
        if self.states_explored > MAX_STATES {
            return Err(format!(
                "{}: oracle_limit_exceeded: more than {MAX_STATES} states",
                self.scenario.id
            ));
        }

        if let Some(decision_class) = self.common_decision_class(belief) {
            let terminal = Policy {
                resolution_class: ResolutionClass::Guaranteed,
                worst_case_cost: Cost::default(),
                tree: json!({
                    "kind": "resolved",
                    "decision_class": decision_class,
                    "remaining_hypotheses": self.hypothesis_ids(belief)
                }),
            };
            let result = vec![terminal];
            self.memo.insert((belief, available_queries), result.clone());
            return Ok(result);
        }

        let mut candidates = vec![Policy {
            resolution_class: ResolutionClass::Unresolved,
            worst_case_cost: Cost::default(),
            tree: json!({
                "kind": "abstain",
                "reason": "material_decision_uncertainty",
                "remaining_hypotheses": self.hypothesis_ids(belief)
            }),
        }];

        for query_index in 0..self.scenario.queries.len() {
            if available_queries & (1u64 << query_index) == 0 {
                continue;
            }
            let query = self.scenario.queries[query_index].clone();
            if !query.permitted || !query.cost.fits(self.budget) {
                continue;
            }
            let remaining_queries = available_queries & !(1u64 << query_index);
            let mut branches: Vec<(String, String, Vec<Policy>)> = Vec::new();

            for outcome in &query.outcomes {
                let outcome_mask = self.mask_for_ids(&outcome.compatible_hypotheses)?;
                let child_belief = belief & outcome_mask;
                if child_belief == 0 {
                    continue;
                }
                let child_frontier = self.solve(child_belief, remaining_queries)?;
                branches.push((outcome.outcome_id.clone(), outcome.kind.clone(), child_frontier));
            }
            if branches.is_empty() {
                continue;
            }
            branches.sort_by(|a, b| a.0.cmp(&b.0));

            let mut combinations = Vec::new();
            self.combine_branch_policies(
                &query,
                &branches,
                0,
                &mut Vec::new(),
                &mut combinations,
            )?;
            candidates.extend(combinations);
        }

        let result = pareto_prune(candidates, self.budget)?;
        self.memo
            .insert((belief, available_queries), result.clone());
        Ok(result)
    }

    fn combine_branch_policies(
        &mut self,
        query: &Query,
        branches: &[(String, String, Vec<Policy>)],
        branch_index: usize,
        selected: &mut Vec<(String, String, Policy)>,
        output: &mut Vec<Policy>,
    ) -> Result<(), String> {
        if branch_index == branches.len() {
            self.policy_combinations_explored += 1;
            if self.policy_combinations_explored > MAX_POLICY_COMBINATIONS {
                return Err(format!(
                    "{}: oracle_limit_exceeded: more than {MAX_POLICY_COMBINATIONS} policy combinations",
                    self.scenario.id
                ));
            }
            let quality = combined_resolution_class(selected);
            let mut worst_child_cost = Cost::default();
            for (_, _, policy) in selected.iter() {
                worst_child_cost = worst_child_cost.component_max(policy.worst_case_cost);
            }
            let total_cost = query.cost.checked_add(worst_child_cost)?;
            if !total_cost.fits(self.budget) {
                return Ok(());
            }
            let branches_json: Vec<Value> = selected
                .iter()
                .map(|(outcome_id, kind, policy)| {
                    json!({
                        "outcome_id": outcome_id,
                        "kind": kind,
                        "policy": &policy.tree
                    })
                })
                .collect();
            output.push(Policy {
                resolution_class: quality,
                worst_case_cost: total_cost,
                tree: json!({
                    "kind": "query",
                    "query_id": query.id,
                    "query_cost": query.cost,
                    "branches": branches_json
                }),
            });
            return Ok(());
        }

        let (outcome_id, kind, child_policies) = &branches[branch_index];
        for child_policy in child_policies {
            selected.push((outcome_id.clone(), kind.clone(), child_policy.clone()));
            self.combine_branch_policies(
                query,
                branches,
                branch_index + 1,
                selected,
                output,
            )?;
            selected.pop();
        }
        Ok(())
    }
}

fn combined_resolution_class(branches: &[(String, String, Policy)]) -> ResolutionClass {
    if branches
        .iter()
        .all(|(_, _, policy)| policy.resolution_class == ResolutionClass::Guaranteed)
    {
        return ResolutionClass::Guaranteed;
    }

    let observed: Vec<ResolutionClass> = branches
        .iter()
        .filter(|(_, kind, _)| kind == "observed")
        .map(|(_, _, policy)| policy.resolution_class)
        .collect();

    if !observed.is_empty()
        && observed.iter().all(|class| {
            matches!(
                class,
                ResolutionClass::Conditional | ResolutionClass::Guaranteed
            )
        })
    {
        ResolutionClass::Conditional
    } else if observed.iter().any(|class| {
        matches!(
            class,
            ResolutionClass::Partial
                | ResolutionClass::Conditional
                | ResolutionClass::Guaranteed
        )
    }) {
        ResolutionClass::Partial
    } else {
        ResolutionClass::Unresolved
    }
}

fn policy_key(policy: &Policy) -> String {
    serde_json::to_string(&policy.tree).unwrap_or_default()
}

fn pareto_prune(mut candidates: Vec<Policy>, budget: Cost) -> Result<Vec<Policy>, String> {
    candidates.retain(|policy| policy.worst_case_cost.fits(budget));
    candidates.sort_by(|a, b| {
        (
            a.resolution_class,
            a.worst_case_cost,
            policy_key(a),
        )
            .cmp(&(
                b.resolution_class,
                b.worst_case_cost,
                policy_key(b),
            ))
    });

    let mut frontier: Vec<Policy> = Vec::new();
    for candidate in candidates {
        if frontier.iter().any(|existing| {
            existing.resolution_class == candidate.resolution_class
                && existing
                    .worst_case_cost
                    .weakly_dominates(candidate.worst_case_cost)
        }) {
            continue;
        }
        frontier.retain(|existing| {
            !(existing.resolution_class == candidate.resolution_class
                && candidate
                    .worst_case_cost
                    .strictly_dominates(existing.worst_case_cost))
        });
        frontier.push(candidate);
        if frontier.len() > MAX_FRONTIER_PER_STATE {
            return Err(format!(
                "oracle_limit_exceeded: frontier exceeds {MAX_FRONTIER_PER_STATE} policies"
            ));
        }
    }
    frontier.sort_by(|a, b| {
        (
            a.resolution_class,
            a.worst_case_cost,
            policy_key(a),
        )
            .cmp(&(
                b.resolution_class,
                b.worst_case_cost,
                policy_key(b),
            ))
    });
    Ok(frontier)
}

fn validate_expected_status(
    scenario: &Scenario,
    frontier: &[Policy],
) -> Result<(), String> {
    let has_guaranteed = frontier
        .iter()
        .any(|policy| policy.resolution_class == ResolutionClass::Guaranteed);
    let has_conditional = frontier
        .iter()
        .any(|policy| policy.resolution_class == ResolutionClass::Conditional);
    let has_resolving_policy = has_guaranteed || has_conditional;

    match scenario.expected_status.as_str() {
        "decision_resolved" => {
            if has_guaranteed {
                Ok(())
            } else {
                Err(format!(
                    "{}: expected a guaranteed decision-resolving policy",
                    scenario.id
                ))
            }
        }
        "conditionally_resolved_unknown_outcome_abstains" => {
            if has_conditional {
                Ok(())
            } else {
                Err(format!(
                    "{}: expected a conditional policy that abstains on unresolved outcomes",
                    scenario.id
                ))
            }
        }
        "stop_same_decision_class" => {
            let decisions: BTreeSet<&str> = scenario
                .hypotheses
                .iter()
                .map(|hypothesis| hypothesis.decision_class.as_str())
                .collect();
            if decisions.len() != 1 {
                return Err(format!(
                    "{}: stop_same_decision_class requires one shared decision class",
                    scenario.id
                ));
            }
            if !has_guaranteed
                || frontier.iter().any(|policy| {
                    policy.resolution_class == ResolutionClass::Guaranteed
                        && policy.worst_case_cost != Cost::default()
                })
            {
                return Err(format!(
                    "{}: shared-decision case must stop with zero evidence-acquisition cost",
                    scenario.id
                ));
            }
            Ok(())
        }
        "abstain_no_permitted_discriminator" | "abstain_budget_exhausted" => {
            if has_resolving_policy {
                Err(format!(
                    "{}: expected abstention, but a resolving policy was found",
                    scenario.id
                ))
            } else {
                Ok(())
            }
        }
        other => Err(format!(
            "{}: unsupported expected_status {other}",
            scenario.id
        )),
    }
}

fn cost_key(cost: Cost) -> (u32, u32, u32) {
    (
        cost.staff_minutes,
        cost.cash_zar_cents,
        cost.delay_hours,
    )
}

fn validate_reference_frontier(
    scenario: &Scenario,
    frontier: &[Policy],
) -> Result<(), String> {
    if scenario.hand_reference_frontier.is_empty() {
        return Ok(());
    }
    let actual: BTreeSet<(u32, u32, u32)> = frontier
        .iter()
        .filter(|policy| policy.resolution_class == ResolutionClass::Conditional)
        .map(|policy| cost_key(policy.worst_case_cost))
        .collect();
    let expected: BTreeSet<(u32, u32, u32)> = scenario
        .hand_reference_frontier
        .iter()
        .map(|reference| cost_key(reference.cost))
        .collect();
    if actual != expected {
        return Err(format!(
            "{}: exact conditional Pareto frontier differs from hand reference; actual={actual:?}, expected={expected:?}",
            scenario.id
        ));
    }
    Ok(())
}

fn verify_values(
    schema_value: &Value,
    corpus_value: &Value,
    fixture_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<Report, String> {
    validate_schema(schema_value, corpus_value)?;
    let mut corpus: Corpus = serde_json::from_value(corpus_value.clone())
        .map_err(|error| format!("fixture does not match corpus types: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let mut receipts = Vec::new();
    for scenario in &corpus.scenarios {
        let mut solver = Solver::new(scenario);
        let initial_hypotheses = solver.all_hypotheses_mask();
        let initial_queries = solver.all_queries_mask();
        let frontier = solver.solve(initial_hypotheses, initial_queries)?;
        validate_expected_status(scenario, &frontier)?;
        validate_reference_frontier(scenario, &frontier)?;

        receipts.push(ScenarioReceipt {
            scenario_id: scenario.id.clone(),
            hypothesis_count: scenario.hypotheses.len(),
            budget: scenario.budget,
            permitted_query_count: scenario.queries.iter().filter(|q| q.permitted).count(),
            states_explored: solver.states_explored,
            policy_combinations_explored: solver.policy_combinations_explored,
            oracle_limit: format!(
                "hypotheses<={MAX_HYPOTHESES}; queries<={MAX_QUERIES}; states<={MAX_STATES}; policy_combinations<={MAX_POLICY_COMBINATIONS}; frontier_per_state<={MAX_FRONTIER_PER_STATE}"
            ),
            frontier,
            expected_contract: scenario.expected_status.clone(),
        });
    }

    Ok(Report {
        status: "SYNTHETIC_ONLY_EXACT_ADAPTIVE_ORACLE_PASS",
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: sha256(fixture_bytes),
        schema_sha256: sha256(schema_bytes),
        scenario_count: receipts.len(),
        scenarios: receipts,
        limitations: vec![
            "All cases and costs are synthetic contract inputs, not measured labour-market facts.",
            "Resolution classes distinguish guaranteed decisions, conditional resolution with abstention, and unresolved cases; there are no invented hypothesis probabilities.",
            "The Pareto frontier minimizes staff effort, cash cost, and delay separately within each resolution class (unresolved, partial observed-branch resolution, conditional resolution with unknown-outcome abstention, or guaranteed).",
            "The exact oracle is bounded and fails closed on any state, combination, or frontier limit.",
            "A solver pass establishes consistency with this synthetic contract only, not field validity, causal impact, or SOTA superiority.",
        ],
        claim_ceiling: corpus.claim_ceiling,
    })
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let fixture_path = args
        .next()
        .unwrap_or_else(|| FIXTURE_DEFAULT.to_string());
    let schema_path = args
        .next()
        .unwrap_or_else(|| SCHEMA_DEFAULT.to_string());

    let fixture_bytes = fs::read(&fixture_path)
        .map_err(|error| format!("cannot read {fixture_path}: {error}"))?;
    let schema_bytes = fs::read(&schema_path)
        .map_err(|error| format!("cannot read {schema_path}: {error}"))?;
    let fixture_value: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("fixture JSON is invalid: {error}"))?;
    let schema_value: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;

    let report = verify_values(
        &schema_value,
        &fixture_value,
        &fixture_bytes,
        &schema_bytes,
    )?;
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
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../docs/research/fixtures/workforce_active_evidence_scenarios_v0_1.json");
    const SCHEMA: &str =
        include_str!("../../../docs/research/schemas/workforce_active_evidence_v0_1.schema.json");

    fn loaded() -> (Value, Value, Vec<u8>, Vec<u8>) {
        let fixture_value: Value = serde_json::from_str(FIXTURE).expect("fixture JSON parses");
        let schema_value: Value = serde_json::from_str(SCHEMA).expect("schema JSON parses");
        (
            schema_value,
            fixture_value,
            FIXTURE.as_bytes().to_vec(),
            SCHEMA.as_bytes().to_vec(),
        )
    }

    fn test_report() -> Report {
        let (schema, fixture, fixture_bytes, schema_bytes) = loaded();
        verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes)
            .expect("synthetic corpus and exact oracle satisfy the contract")
    }

    fn find_scenario<'a>(report: &'a Report, id: &str) -> &'a ScenarioReceipt {
        report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario_id == id)
            .expect("scenario exists")
    }

    fn collect_query_ids(tree: &Value, output: &mut Vec<String>) {
        if tree["kind"] == "query" {
            if let Some(id) = tree["query_id"].as_str() {
                output.push(id.to_string());
            }
            if let Some(branches) = tree["branches"].as_array() {
                for branch in branches {
                    collect_query_ids(&branch["policy"], output);
                }
            }
        }
    }

    #[test]
    fn strict_schema_and_exact_oracle_accept_frozen_synthetic_corpus() {
        let report = test_report();
        assert_eq!(report.status, "SYNTHETIC_ONLY_EXACT_ADAPTIVE_ORACLE_PASS");
        assert_eq!(report.scenario_count, 5);
        assert!(report.scenarios.iter().all(|scenario| {
            scenario.frontier.iter().all(|policy| policy.worst_case_cost.fits(scenario.budget))
        }));
    }

    #[test]
    fn exact_oracle_matches_hand_calculated_multi_resource_frontier() {
        let report = test_report();
        let scenario = find_scenario(&report, "AE-001");
        let mut costs: Vec<(u32, u32, u32)> = scenario
            .frontier
            .iter()
            .filter(|policy| policy.resolution_class == ResolutionClass::Conditional)
            .map(|policy| cost_key(policy.worst_case_cost))
            .collect();
        costs.sort();
        assert_eq!(costs, vec![(30, 5000, 1), (70, 0, 9)]);
        assert!(scenario.frontier.iter().any(|policy| {
            policy.resolution_class == ResolutionClass::Partial
                && policy.worst_case_cost == Cost {
                    staff_minutes: 10,
                    cash_zar_cents: 5000,
                    delay_hours: 0
                }
        }));
    }

    #[test]
    fn same_decision_class_stops_without_buying_more_information() {
        let report = test_report();
        let scenario = find_scenario(&report, "AE-002");
        assert_eq!(scenario.frontier.len(), 1);
        assert_eq!(
            scenario.frontier[0].resolution_class,
            ResolutionClass::Guaranteed
        );
        assert_eq!(scenario.frontier[0].worst_case_cost, Cost::default());
        assert_eq!(scenario.frontier[0].tree["kind"], "resolved");
    }

    #[test]
    fn forbidden_queries_never_appear_in_any_policy_tree() {
        let report = test_report();
        let mut selected = Vec::new();
        for scenario in &report.scenarios {
            for policy in &scenario.frontier {
                collect_query_ids(&policy.tree, &mut selected);
            }
        }
        assert!(!selected.iter().any(|id| {
            id == "Q_RESTRICTED_JOIN" || id == "Q_PERSONAL_BANK_LOOKUP"
        }));
    }

    #[test]
    fn budget_exhaustion_and_no_discriminator_abstain() {
        let report = test_report();
        for id in ["AE-003", "AE-005"] {
            let scenario = find_scenario(&report, id);
            assert!(scenario
                .frontier
                .iter()
                .all(|policy| policy.resolution_class == ResolutionClass::Unresolved));
        }
        let ae005 = find_scenario(&report, "AE-005");
        assert!(ae005.frontier.iter().all(|policy| {
            policy.tree["kind"] == "abstain"
        }));
    }

    #[test]
    fn row_permutations_preserve_scenario_receipts_and_frontiers() {
        let (schema, fixture, fixture_bytes, schema_bytes) = loaded();
        let first = verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes)
            .expect("original corpus verifies");

        let mut permuted = fixture.clone();
        let scenarios = permuted["scenarios"]
            .as_array_mut()
            .expect("scenarios array");
        scenarios.reverse();
        for scenario in scenarios {
            scenario["hypotheses"]
                .as_array_mut()
                .expect("hypotheses array")
                .reverse();
            scenario["queries"]
                .as_array_mut()
                .expect("queries array")
                .reverse();
            for query in scenario["queries"].as_array_mut().expect("queries array") {
                query["outcomes"]
                    .as_array_mut()
                    .expect("outcomes array")
                    .reverse();
            }
        }
        let second = verify_values(&schema, &permuted, &fixture_bytes, &schema_bytes)
            .expect("permuted corpus verifies");
        assert_eq!(first.scenarios, second.scenarios);
    }

    #[test]
    fn schema_rejects_a_fixture_that_claims_to_be_observed() {
        let (schema, mut fixture, fixture_bytes, schema_bytes) = loaded();
        fixture["synthetic_only"] = Value::Bool(false);
        assert!(verify_values(&schema, &fixture, &fixture_bytes, &schema_bytes).is_err());
    }
}
