// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Proof-carrying selection over a supplied finite dispatch frontier.
//!
//! The witness proves why one supplied alternative outranks every other
//! alternative in that frontier under a versioned policy. It does not prove
//! that the frontier contains every feasible schedule. That separate claim
//! remains the responsibility of the exact oracle plus independent audit.

use crate::energy_dispatch::{
    DispatchIntervalDecision, DispatchObjective,
    EnergyDispatchAssessment, EnergyDispatchRequest,
};
use crate::energy_dispatch_policy::{
    ASSESSMENT_SCHEMA_VERSION, DispatchObjectiveId, DispatchSelectionError,
    DispatchSelectionPolicy, ObjectiveDirection, select_dispatch_alternative,
};
use crate::energy_dispatch_verifier::verify_dispatch_schedule;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

pub const DECISION_WITNESS_SCHEMA_VERSION: &str = "energy-dispatch-decision-witness-v0.1";
pub const WITNESS_CLAIM_CEILING: &str = "proves the selected alternative outranks every other member of the supplied frontier under the bound policy; does not prove frontier completeness, physical model validity, evidence authenticity, operational safety or dispatch authorization";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChallengerProofBasis {
    FirstDifferingObjectiveTier { tier_index: usize },
    StableScheduleTieBreakV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengerProof {
    pub challenger_frontier_index: usize,
    pub challenger_schedule_sha256: String,
    pub basis: ChallengerProofBasis,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchDecisionWitness {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_sha256: String,
    pub request_sha256: String,
    pub frontier_sha256: String,
    pub frontier_count: usize,
    pub selected_frontier_index: usize,
    pub selected_objective: DispatchObjective,
    pub selected_schedule_sha256: String,
    pub stable_tie_break_applied: bool,
    /// Exactly one proof for every non-selected frontier member, in frontier order.
    pub challenger_proofs: Vec<ChallengerProof>,
    pub claim_ceiling: String,
}

#[derive(Serialize)]
struct PolicyTierIdentity {
    objective: &'static str,
    direction: &'static str,
    aggregation: &'static str,
}

#[derive(Serialize)]
struct PolicyIdentity<'a> {
    schema_version: &'a str,
    policy_id: &'a str,
    objective_tiers: Vec<PolicyTierIdentity>,
    tie_break: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StableStepKey {
    interval_id: String,
    start_unix_seconds: i64,
    duration_seconds: u64,
    grid_outage: bool,
    pv_available_quanta: u16,
    pv_used_quanta: u16,
    pv_curtailed_quanta: u16,
    grid_import_quanta: u16,
    battery_soc_before_quanta: u16,
    battery_charge_input_quanta: u16,
    battery_soc_withdrawal_quanta: u16,
    battery_discharge_delivered_quanta: u16,
    battery_soc_after_quanta: u16,
    essential_demand_quanta: u16,
    essential_served_quanta: u16,
    essential_unserved_quanta: u16,
    flexible_demand_quanta: u16,
    flexible_served_quanta: u16,
    flexible_unserved_quanta: u16,
    grid_cost_units_per_quantum: u32,
    grid_carbon_grams_per_quantum: u32,
    bus_energy_balance_residual_quanta: i64,
}

fn digest<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("canonical witness input serialization failed: {error}"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn policy_identity(policy: &DispatchSelectionPolicy) -> PolicyIdentity<'_> {
    let objective_tiers = policy.objective_tiers.iter().map(|tier| {
        let objective = match tier.objective {
            DispatchObjectiveId::EssentialUnservedQuanta => "essential_unserved_quanta",
            DispatchObjectiveId::FlexibleUnservedQuanta => "flexible_unserved_quanta",
            DispatchObjectiveId::OperatingCostUnits => "operating_cost_units",
            DispatchObjectiveId::GridCarbonGrams => "grid_carbon_grams",
            DispatchObjectiveId::CurtailedPvQuanta => "curtailed_pv_quanta",
            DispatchObjectiveId::TerminalSocQuanta => "terminal_soc_quanta",
        };
        let direction = match tier.direction {
            ObjectiveDirection::Minimize => "minimize",
            ObjectiveDirection::Maximize => "maximize",
        };
        let aggregation = match tier.aggregation {
            crate::energy_dispatch_policy::ObjectiveAggregation::HorizonTotal => "horizon_total",
            crate::energy_dispatch_policy::ObjectiveAggregation::TerminalValue => "terminal_value",
        };
        PolicyTierIdentity { objective, direction, aggregation }
    }).collect();
    let tie_break = match &policy.tie_break {
        crate::energy_dispatch_policy::DispatchTieBreak::StableScheduleV1 => "stable_schedule_v1",
        crate::energy_dispatch_policy::DispatchTieBreak::Unsupported(_) => "unsupported",
    };
    PolicyIdentity {
        schema_version: &policy.schema_version,
        policy_id: &policy.policy_id,
        objective_tiers,
        tie_break,
    }
}

fn stable_step_key(step: &DispatchIntervalDecision) -> StableStepKey {
    StableStepKey {
        interval_id: step.interval_id.clone(),
        start_unix_seconds: step.start_unix_seconds,
        duration_seconds: step.duration_seconds,
        grid_outage: step.grid_outage,
        pv_available_quanta: step.pv_available_quanta,
        pv_used_quanta: step.pv_used_quanta,
        pv_curtailed_quanta: step.pv_curtailed_quanta,
        grid_import_quanta: step.grid_import_quanta,
        battery_soc_before_quanta: step.battery_soc_before_quanta,
        battery_charge_input_quanta: step.battery_charge_input_quanta,
        battery_soc_withdrawal_quanta: step.battery_soc_withdrawal_quanta,
        battery_discharge_delivered_quanta: step.battery_discharge_delivered_quanta,
        battery_soc_after_quanta: step.battery_soc_after_quanta,
        essential_demand_quanta: step.essential_demand_quanta,
        essential_served_quanta: step.essential_served_quanta,
        essential_unserved_quanta: step.essential_unserved_quanta,
        flexible_demand_quanta: step.flexible_demand_quanta,
        flexible_served_quanta: step.flexible_served_quanta,
        flexible_unserved_quanta: step.flexible_unserved_quanta,
        grid_cost_units_per_quantum: step.grid_cost_units_per_quantum,
        grid_carbon_grams_per_quantum: step.grid_carbon_grams_per_quantum,
        bus_energy_balance_residual_quanta: step.bus_energy_balance_residual_quanta,
    }
}

fn stable_schedule_key(schedule: &[DispatchIntervalDecision]) -> Vec<StableStepKey> {
    schedule.iter().map(stable_step_key).collect()
}

fn objective_value(objective: &DispatchObjective, id: DispatchObjectiveId) -> u64 {
    match id {
        DispatchObjectiveId::EssentialUnservedQuanta => objective.essential_unserved_quanta,
        DispatchObjectiveId::FlexibleUnservedQuanta => objective.flexible_unserved_quanta,
        DispatchObjectiveId::OperatingCostUnits => objective.operating_cost_units,
        DispatchObjectiveId::GridCarbonGrams => objective.grid_carbon_grams,
        DispatchObjectiveId::CurtailedPvQuanta => objective.curtailed_pv_quanta,
        DispatchObjectiveId::TerminalSocQuanta => u64::from(objective.terminal_soc_quanta),
    }
}

/// Returns the first policy tier that distinguishes the two vectors.
/// The Ordering is from the perspective of the left/winning alternative.
fn first_objective_difference(
    left: &DispatchObjective,
    right: &DispatchObjective,
    policy: &DispatchSelectionPolicy,
) -> Option<(usize, Ordering)> {
    for (tier_index, tier) in policy.objective_tiers.iter().enumerate() {
        let raw = objective_value(left, tier.objective).cmp(&objective_value(right, tier.objective));
        let comparison = match tier.direction {
            ObjectiveDirection::Minimize => raw,
            ObjectiveDirection::Maximize => raw.reverse(),
        };
        if comparison != Ordering::Equal {
            return Some((tier_index, comparison));
        }
    }
    None
}

fn objective_weakly_dominates(left: &DispatchObjective, right: &DispatchObjective) -> bool {
    left.essential_unserved_quanta <= right.essential_unserved_quanta
        && left.flexible_unserved_quanta <= right.flexible_unserved_quanta
        && left.operating_cost_units <= right.operating_cost_units
        && left.grid_carbon_grams <= right.grid_carbon_grams
        && left.curtailed_pv_quanta <= right.curtailed_pv_quanta
        && left.terminal_soc_quanta >= right.terminal_soc_quanta
}

fn recompute_objective_independently(
    request: &EnergyDispatchRequest,
    schedule: &[DispatchIntervalDecision],
) -> Result<DispatchObjective, String> {
    if schedule.len() != request.intervals.len() {
        return Err("schedule length does not match request interval count".to_owned());
    }
    let mut essential_unserved = 0_u64;
    let mut flexible_unserved = 0_u64;
    let mut operating_cost = 0_u64;
    let mut grid_carbon = 0_u64;
    let mut curtailed_pv = 0_u64;

    for (step, input) in schedule.iter().zip(&request.intervals) {
        let essential_shortfall = input.essential_demand_quanta
            .checked_sub(step.essential_served_quanta)
            .ok_or_else(|| "essential service exceeds input demand".to_owned())?;
        let flexible_shortfall = input.flexible_demand_quanta
            .checked_sub(step.flexible_served_quanta)
            .ok_or_else(|| "flexible service exceeds input demand".to_owned())?;
        let pv_spill = input.pv_available_quanta
            .checked_sub(step.pv_used_quanta)
            .ok_or_else(|| "PV use exceeds input availability".to_owned())?;

        essential_unserved = essential_unserved.checked_add(u64::from(essential_shortfall))
            .ok_or_else(|| "essential shortfall sum overflow".to_owned())?;
        flexible_unserved = flexible_unserved.checked_add(u64::from(flexible_shortfall))
            .ok_or_else(|| "flexible shortfall sum overflow".to_owned())?;
        operating_cost = operating_cost.checked_add(
            u64::from(step.grid_import_quanta)
                .checked_mul(u64::from(input.grid_cost_units_per_quantum))
                .ok_or_else(|| "interval operating cost overflow".to_owned())?
        ).ok_or_else(|| "operating cost sum overflow".to_owned())?;
        grid_carbon = grid_carbon.checked_add(
            u64::from(step.grid_import_quanta)
                .checked_mul(u64::from(input.grid_carbon_grams_per_quantum))
                .ok_or_else(|| "interval grid carbon overflow".to_owned())?
        ).ok_or_else(|| "grid carbon sum overflow".to_owned())?;
        curtailed_pv = curtailed_pv.checked_add(u64::from(pv_spill))
            .ok_or_else(|| "PV curtailment sum overflow".to_owned())?;
    }

    let terminal_soc = schedule.last()
        .map(|last| last.battery_soc_after_quanta)
        .unwrap_or(request.battery.initial_soc_quanta);

    Ok(DispatchObjective {
        essential_unserved_quanta: essential_unserved,
        flexible_unserved_quanta: flexible_unserved,
        operating_cost_units: operating_cost,
        grid_carbon_grams: grid_carbon,
        curtailed_pv_quanta: curtailed_pv,
        terminal_soc_quanta: terminal_soc,
    })
}

fn check_assessment_binding(
    request: &EnergyDispatchRequest,
    assessment: &EnergyDispatchAssessment,
) -> Result<(), String> {
    if assessment.assessment_schema_version != ASSESSMENT_SCHEMA_VERSION {
        return Err("assessment schema version is unsupported".to_owned());
    }
    if assessment.case_id != request.case_id {
        return Err("assessment case ID does not match request".to_owned());
    }
    if assessment.synthetic != request.synthetic
        || assessment.evidence_class != request.evidence_class
        || assessment.evidence_ref != request.evidence_ref
        || assessment.evidence_as_of != request.evidence_as_of
        || assessment.energy_quantum_kwh.to_bits() != request.energy_quantum_kwh.to_bits()
    {
        return Err("assessment metadata does not match the exact request".to_owned());
    }
    if !assessment.search_is_exact || !assessment.pareto_frontier_complete {
        return Err("assessment does not declare an exact complete frontier".to_owned());
    }
    if assessment.pareto_frontier_count != assessment.pareto_alternatives.len()
        || assessment.pareto_alternatives.is_empty()
    {
        return Err("assessment frontier count is empty or inconsistent".to_owned());
    }
    Ok(())
}

/// Build a witness after the existing selector validates and selects from the
/// complete supplied frontier. Every non-winner is represented exactly once.
pub fn create_dispatch_decision_witness(
    policy: &DispatchSelectionPolicy,
    request: &EnergyDispatchRequest,
    assessment: &EnergyDispatchAssessment,
) -> Result<DispatchDecisionWitness, String> {
    let selection = select_dispatch_alternative(policy, request, assessment)
        .map_err(|error: DispatchSelectionError| format!("selection failed: {error:?}"))?;
    check_assessment_binding(request, assessment)?;

    let winner = assessment.pareto_alternatives
        .get(selection.selected_frontier_index)
        .ok_or_else(|| "selector returned an index outside the frontier".to_owned())?;

    let mut challenger_proofs = Vec::with_capacity(assessment.pareto_alternatives.len().saturating_sub(1));
    let mut stable_tie_break_applied = false;
    for (index, challenger) in assessment.pareto_alternatives.iter().enumerate() {
        if index == selection.selected_frontier_index { continue; }
        let basis = match first_objective_difference(&winner.objective, &challenger.objective, policy) {
            Some((tier_index, Ordering::Less)) => {
                ChallengerProofBasis::FirstDifferingObjectiveTier { tier_index }
            }
            Some((_tier_index, Ordering::Greater)) => {
                return Err(format!("selected alternative loses to frontier member {index}"));
            }
            Some((_tier_index, Ordering::Equal)) => unreachable!("a differing tier cannot compare equal"),
            None => {
                if stable_schedule_key(&winner.schedule) >= stable_schedule_key(&challenger.schedule) {
                    return Err(format!("selected alternative does not win stable tie-break against {index}"));
                }
                stable_tie_break_applied = true;
                ChallengerProofBasis::StableScheduleTieBreakV1
            }
        };
        challenger_proofs.push(ChallengerProof {
            challenger_frontier_index: index,
            challenger_schedule_sha256: digest(&challenger.schedule)?,
            basis,
        });
    }

    Ok(DispatchDecisionWitness {
        schema_version: DECISION_WITNESS_SCHEMA_VERSION.to_owned(),
        policy_id: policy.policy_id.clone(),
        policy_sha256: digest(&policy_identity(policy))?,
        request_sha256: digest(request)?,
        frontier_sha256: digest(&assessment.pareto_alternatives)?,
        frontier_count: assessment.pareto_alternatives.len(),
        selected_frontier_index: selection.selected_frontier_index,
        selected_objective: winner.objective.clone(),
        selected_schedule_sha256: digest(&winner.schedule)?,
        stable_tie_break_applied,
        challenger_proofs,
        claim_ceiling: WITNESS_CLAIM_CEILING.to_owned(),
    })
}

/// Verify a witness without calling the selector. The verifier independently
/// checks every candidate, recomputes objective vectors from request data,
/// binds policy/request/frontier digests, and checks the first decisive tier or
/// stable tie-break for every challenger.
pub fn verify_dispatch_decision_witness(
    policy: &DispatchSelectionPolicy,
    request: &EnergyDispatchRequest,
    assessment: &EnergyDispatchAssessment,
    witness: &DispatchDecisionWitness,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if let Err(error) = policy.validate() {
        errors.push(format!("policy validation failed: {error:?}"));
    }
    if witness.schema_version != DECISION_WITNESS_SCHEMA_VERSION {
        errors.push("unsupported decision witness schema version".to_owned());
    }
    if witness.claim_ceiling != WITNESS_CLAIM_CEILING {
        errors.push("witness claim ceiling was modified".to_owned());
    }
    if witness.policy_id != policy.policy_id {
        errors.push("witness policy ID does not match policy".to_owned());
    }
    match digest(&policy_identity(policy)) {
        Ok(expected) if expected == witness.policy_sha256 => {}
        Ok(_) => errors.push("policy digest mismatch".to_owned()),
        Err(error) => errors.push(error),
    }
    match digest(request) {
        Ok(expected) if expected == witness.request_sha256 => {}
        Ok(_) => errors.push("request digest mismatch".to_owned()),
        Err(error) => errors.push(error),
    }
    if let Err(error) = check_assessment_binding(request, assessment) {
        errors.push(error);
    }
    match digest(&assessment.pareto_alternatives) {
        Ok(expected) if expected == witness.frontier_sha256 => {}
        Ok(_) => errors.push("frontier digest mismatch".to_owned()),
        Err(error) => errors.push(error),
    }

    let alternatives = &assessment.pareto_alternatives;
    if witness.frontier_count != alternatives.len() {
        errors.push("witness frontier count does not match assessment".to_owned());
    }
    if alternatives.len() > crate::energy_dispatch::MAX_PARETO_FRONTIER {
        errors.push("assessment frontier exceeds the versioned maximum".to_owned());
    }
    let Some(winner) = alternatives.get(witness.selected_frontier_index) else {
        errors.push("selected frontier index is out of range".to_owned());
        return Err(errors);
    };
    if winner.objective != witness.selected_objective {
        errors.push("selected objective does not match frontier member".to_owned());
    }
    match digest(&winner.schedule) {
        Ok(expected) if expected == witness.selected_schedule_sha256 => {}
        Ok(_) => errors.push("selected schedule digest mismatch".to_owned()),
        Err(error) => errors.push(error),
    }

    // The witness verifier also checks frontier internal consistency; it does not
    // rely on the producer's claim that the supplied members are non-dominated.
    for left in 0..alternatives.len() {
        for right in (left + 1)..alternatives.len() {
            if objective_weakly_dominates(&alternatives[left].objective, &alternatives[right].objective)
                || objective_weakly_dominates(&alternatives[right].objective, &alternatives[left].objective) {
                errors.push(format!("frontier members {left} and {right} are duplicated or dominated"));
            }
        }
    }

    // Validate each member and recompute the entire objective independently.
    for (index, alternative) in alternatives.iter().enumerate() {
        if let Err(schedule_errors) = verify_dispatch_schedule(request, &alternative.schedule) {
            errors.push(format!("frontier member {index} schedule invalid: {schedule_errors:?}"));
            continue;
        }
        match recompute_objective_independently(request, &alternative.schedule) {
            Ok(recomputed) if recomputed == alternative.objective => {}
            Ok(_) => errors.push(format!("frontier member {index} objective does not match independent recomputation")),
            Err(error) => errors.push(format!("frontier member {index} objective recomputation failed: {error}")),
        }
    }

    let expected_proof_count = alternatives.len().saturating_sub(1);
    if witness.challenger_proofs.len() != expected_proof_count {
        errors.push(format!(
            "challenger proof coverage mismatch: expected {expected_proof_count}, got {}",
            witness.challenger_proofs.len()
        ));
    }

    let mut expected_index = 0_usize;
    let mut observed_stable_tie_break = false;
    for proof in &witness.challenger_proofs {
        while expected_index == witness.selected_frontier_index {
            expected_index += 1;
        }
        if proof.challenger_frontier_index != expected_index {
            errors.push(format!(
                "challenger coverage/order mismatch: expected index {expected_index}, got {}",
                proof.challenger_frontier_index
            ));
            expected_index = proof.challenger_frontier_index;
        }
        let index = proof.challenger_frontier_index;
        expected_index = expected_index.saturating_add(1);
        let Some(challenger) = alternatives.get(index) else {
            errors.push(format!("challenger index {index} is out of range"));
            continue;
        };
        match digest(&challenger.schedule) {
            Ok(expected) if expected == proof.challenger_schedule_sha256 => {}
            Ok(_) => errors.push(format!("challenger {index} schedule digest mismatch")),
            Err(error) => errors.push(error),
        }
        match first_objective_difference(&winner.objective, &challenger.objective, policy) {
            Some((tier_index, Ordering::Less)) => {
                if proof.basis != ChallengerProofBasis::FirstDifferingObjectiveTier { tier_index } {
                    errors.push(format!("challenger {index} proof does not name its first decisive tier"));
                }
            }
            Some((_tier_index, Ordering::Greater)) => {
                errors.push(format!("selected alternative loses to challenger {index}"));
            }
            Some((_tier_index, Ordering::Equal)) => {
                errors.push(format!("challenger {index} has an invalid objective comparison"));
            }
            None => {
                observed_stable_tie_break = true;
                if proof.basis != ChallengerProofBasis::StableScheduleTieBreakV1 {
                    errors.push(format!("challenger {index} omits the stable tie-break proof"));
                }
                if stable_schedule_key(&winner.schedule) >= stable_schedule_key(&challenger.schedule) {
                    errors.push(format!("selected schedule does not precede challenger {index} under stable key"));
                }
            }
        }
    }
    if observed_stable_tie_break != witness.stable_tie_break_applied {
        errors.push("stable_tie_break_applied flag does not match challenger evidence".to_owned());
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_dispatch::{EnergyDispatchRequest, assess_energy_dispatch};
    use crate::energy_dispatch_policy::DispatchSelectionPolicy;

    const FIXTURE: &str =
        include_str!("../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    fn fixture() -> (EnergyDispatchRequest, EnergyDispatchAssessment) {
        let request: EnergyDispatchRequest =
            serde_json::from_str(FIXTURE).expect("frozen request parses");
        let assessment =
            assess_energy_dispatch(&request).expect("frozen request is feasible");
        (request, assessment)
    }

    #[test]
    fn witness_covers_and_verifies_every_frontier_challenger() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        assert_eq!(witness.challenger_proofs.len(), assessment.pareto_alternatives.len() - 1);
        verify_dispatch_decision_witness(&policy, &request, &assessment, &witness)
            .expect("independent witness verifier accepts");
    }

    #[test]
    fn serialized_witness_round_trips_and_still_verifies() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        let bytes = serde_json::to_vec(&witness).expect("witness serializes");
        let decoded: DispatchDecisionWitness =
            serde_json::from_slice(&bytes).expect("witness deserializes strictly");
        verify_dispatch_decision_witness(&policy, &request, &assessment, &decoded)
            .expect("round-tripped witness verifies");
    }

    #[test]
    fn verifier_rejects_missing_challenger_evidence() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let mut witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        witness.challenger_proofs.pop();
        let errors = verify_dispatch_decision_witness(&policy, &request, &assessment, &witness)
            .expect_err("omitted challenger is rejected");
        assert!(errors.iter().any(|error| error.contains("coverage mismatch")));
    }

    #[test]
    fn verifier_rejects_wrong_first_decisive_tier() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let mut witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        let proof = witness.challenger_proofs.iter_mut().find(|proof| {
            matches!(&proof.basis, ChallengerProofBasis::FirstDifferingObjectiveTier { .. })
        }).expect("fixture has a challenger separated by an objective tier");
        let original_tier = match proof.basis {
            ChallengerProofBasis::FirstDifferingObjectiveTier { tier_index } => tier_index,
            ChallengerProofBasis::StableScheduleTieBreakV1 => unreachable!("the search filters for an objective-tier proof"),
        };
        proof.basis = ChallengerProofBasis::FirstDifferingObjectiveTier {
            tier_index: (original_tier + 1) % policy.objective_tiers.len(),
        };
        let errors = verify_dispatch_decision_witness(&policy, &request, &assessment, &witness)
            .expect_err("wrong decisive tier is rejected");
        assert!(errors.iter().any(|error| error.contains("first decisive tier")));
    }

    #[test]
    fn verifier_rejects_frontier_and_request_substitution() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        let mut altered = assessment.clone();
        altered.pareto_alternatives.reverse();
        assert!(verify_dispatch_decision_witness(&policy, &request, &altered, &witness)
            .unwrap_err().iter().any(|error| error.contains("frontier digest")));
        let mut altered_request = request.clone();
        altered_request.energy_quantum_kwh *= 2.0;
        assert!(verify_dispatch_decision_witness(&policy, &altered_request, &assessment, &witness)
            .unwrap_err().iter().any(|error| error.contains("request digest")));
    }

    #[test]
    fn witness_binds_policy_order_and_stable_schedule_rule() {
        let (request, assessment) = fixture();
        let policy = DispatchSelectionPolicy::critical_service_first_v1();
        let witness = create_dispatch_decision_witness(&policy, &request, &assessment)
            .expect("selection witness generated");
        let mut altered_policy = policy.clone();
        altered_policy.objective_tiers.swap(1, 2);
        assert!(verify_dispatch_decision_witness(&altered_policy, &request, &assessment, &witness)
            .unwrap_err().iter().any(|error| error.contains("policy digest")));
    }
}
