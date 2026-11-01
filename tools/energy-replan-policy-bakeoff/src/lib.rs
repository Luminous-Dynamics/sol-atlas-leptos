#![forbid(unsafe_code)]
//! Synthetic known-truth comparison of energy-dispatch replanning triggers.
//!
//! Policies receive only the current observation. Hidden event truth and
//! counterfactual outcomes are held by the evaluator and never passed to a
//! policy. This benchmark does not optimize dispatch or authorize actuation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryStatus {
    Fresh,
    Stale,
    Missing,
    Conflicting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionObservation {
    pub step: u32,
    pub telemetry_status: TelemetryStatus,
    /// Forecast-versus-observation residual available at this step.
    pub forecast_residual_kw: u32,
    /// Declared upper uncertainty bound for the current forecast.
    pub uncertainty_bound_kw: u32,
    /// Current estimated usable energy above the hard reserve floor.
    pub reserve_margin_mwh: u32,
    /// Model-estimated avoidable service loss if a new schedule is made.
    pub estimated_avoidable_loss_mwh: u32,
    /// Replanning compute cost in the same synthetic accounting units.
    pub estimated_solver_cost_mwh: u32,
    /// Estimated switching/wear cost of changing the current schedule.
    pub estimated_switching_cost_mwh: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplanAction {
    KeepPlan,
    RequestReplan,
    Abstain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyKind {
    FixedCadence,
    ForecastResidual,
    ForecastUncertainty,
    DecisionValue,
    HybridSafetyOverride,
}

impl PolicyKind {
    pub const ALL: [Self; 5] = [
        Self::FixedCadence,
        Self::ForecastResidual,
        Self::ForecastUncertainty,
        Self::DecisionValue,
        Self::HybridSafetyOverride,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::FixedCadence => "fixed-cadence",
            Self::ForecastResidual => "forecast-residual",
            Self::ForecastUncertainty => "forecast-uncertainty",
            Self::DecisionValue => "decision-value",
            Self::HybridSafetyOverride => "hybrid-safety",
        }
    }

    /// Decide using only one time-local observation.
    ///
    /// Telemetry quality is enforced by run_policy before this method is
    /// called. Hidden evaluator truth is not an argument and cannot leak
    /// through this interface.
    fn decide(self, observation: &DecisionObservation) -> ReplanAction {
        let estimated_cost = observation
            .estimated_solver_cost_mwh
            .saturating_add(observation.estimated_switching_cost_mwh);
        let positive_estimated_value = observation.estimated_avoidable_loss_mwh > estimated_cost;

        match self {
            Self::FixedCadence => {
                if observation.step > 0 && observation.step.is_multiple_of(3) {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            }
            Self::ForecastResidual => {
                if observation.forecast_residual_kw >= 5_000 {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            }
            Self::ForecastUncertainty => {
                if observation.uncertainty_bound_kw >= 6_000 {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            }
            Self::DecisionValue => {
                if positive_estimated_value {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            }
            Self::HybridSafetyOverride => {
                let hard_reserve_override = observation.reserve_margin_mwh <= 2_000;
                let corroborated_value_trigger = positive_estimated_value
                    && (observation.forecast_residual_kw >= 2_500
                        || observation.uncertainty_bound_kw >= 3_000);
                if hard_reserve_override || corroborated_value_trigger {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaterialEvent {
    pub observable_at_step: u32,
    pub response_deadline_step: u32,
    pub service_loss_without_replan_mwh: u64,
    pub service_loss_with_timely_replan_mwh: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolveOutcomeTruth {
    Completed { completed_at_step: u32 },
    TimedOut,
    Infeasible,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditOutcomeTruth {
    Accepted,
    Rejected,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplanExecutionTruth {
    pub requested_at_step: u32,
    pub solve_outcome: SolveOutcomeTruth,
    pub audit_outcome: AuditOutcomeTruth,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScenarioTruth {
    pub material_event: Option<MaterialEvent>,
    /// Evaluator-only execution outcomes, keyed to an exact replan request step.
    /// These are never exposed to the policy decision function.
    pub replan_executions: Vec<ReplanExecutionTruth>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub observations: Vec<DecisionObservation>,
    /// Evaluator-only state. Never passed to PolicyKind::decide.
    pub truth: ScenarioTruth,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRun {
    pub policy: PolicyKind,
    pub scenario_id: String,
    pub actions: Vec<ReplanAction>,
    pub event_count: u32,
    pub timely_detections: u32,
    pub missed_events: u32,
    pub false_triggers: u32,
    pub redundant_triggers: u32,
    pub solver_invocations: u32,
    pub solver_completed: u32,
    pub solver_failures: u32,
    pub late_solver_completions: u32,
    pub auditor_rejections: u32,
    pub auditor_unavailable: u32,
    pub accepted_replans: u32,
    pub abstentions: u32,
    pub service_loss_mwh: u64,
    pub replanning_cost_mwh: u64,
    pub total_penalty_mwh: u64,
    pub audited_schedule_latency_steps: Option<u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AggregateMetrics {
    pub scenario_count: u32,
    pub event_count: u32,
    pub timely_detections: u32,
    pub missed_events: u32,
    pub false_triggers: u32,
    pub redundant_triggers: u32,
    pub solver_invocations: u32,
    pub solver_completed: u32,
    pub solver_failures: u32,
    pub late_solver_completions: u32,
    pub auditor_rejections: u32,
    pub auditor_unavailable: u32,
    pub accepted_replans: u32,
    pub abstentions: u32,
    pub service_loss_mwh: u64,
    pub replanning_cost_mwh: u64,
    pub total_penalty_mwh: u64,
    pub audited_schedule_latency_step_sum: u64,
    pub audited_schedule_latency_event_count: u32,
}

impl AggregateMetrics {
    pub fn mean_audited_schedule_latency_steps(&self) -> Option<f64> {
        (self.audited_schedule_latency_event_count > 0).then(|| {
            self.audited_schedule_latency_step_sum as f64
                / f64::from(self.audited_schedule_latency_event_count)
        })
    }
}

fn evaluate_run(
    policy: PolicyKind,
    scenario: &Scenario,
    actions: Vec<ReplanAction>,
) -> PolicyRun {
    let replan_steps: Vec<u32> = scenario
        .observations
        .iter()
        .zip(&actions)
        .filter_map(|(observation, action)| {
            (*action == ReplanAction::RequestReplan).then_some(observation.step)
        })
        .collect();

    let abstentions = u32::try_from(
        actions
            .iter()
            .filter(|action| **action == ReplanAction::Abstain)
            .count(),
    )
    .unwrap_or(u32::MAX);
    let solver_invocations = u32::try_from(replan_steps.len()).unwrap_or(u32::MAX);
    let replanning_cost_mwh = scenario
        .observations
        .iter()
        .zip(&actions)
        .filter(|(_, action)| **action == ReplanAction::RequestReplan)
        .map(|(observation, _)| {
            u64::from(observation.estimated_solver_cost_mwh)
                + u64::from(observation.estimated_switching_cost_mwh)
        })
        .sum();

    let mut solver_completed = 0u32;
    let mut solver_failures = 0u32;
    let mut late_solver_completions = 0u32;
    let mut auditor_rejections = 0u32;
    let mut auditor_unavailable = 0u32;
    let mut accepted_replans = 0u32;
    let mut audited_replans = Vec::<(u32, u32)>::new();

    for requested_at_step in replan_steps.iter().copied() {
        let mut matching = scenario
            .truth
            .replan_executions
            .iter()
            .filter(|entry| entry.requested_at_step == requested_at_step);
        let Some(execution) = matching.next() else {
            solver_failures += 1;
            continue;
        };
        if matching.next().is_some() {
            // Ambiguous evaluator provenance cannot establish an execution.
            solver_failures += 1;
            continue;
        }

        match execution.solve_outcome {
            SolveOutcomeTruth::Completed { completed_at_step }
                if completed_at_step >= requested_at_step =>
            {
                solver_completed += 1;
                if let Some(event) = scenario.truth.material_event {
                    if requested_at_step >= event.observable_at_step
                        && completed_at_step > event.response_deadline_step
                    {
                        late_solver_completions += 1;
                    }
                }

                match execution.audit_outcome {
                    AuditOutcomeTruth::Accepted => {
                        accepted_replans += 1;
                        audited_replans.push((requested_at_step, completed_at_step));
                    }
                    AuditOutcomeTruth::Rejected => auditor_rejections += 1,
                    AuditOutcomeTruth::Unavailable => auditor_unavailable += 1,
                }
            }
            SolveOutcomeTruth::Completed { .. }
            | SolveOutcomeTruth::TimedOut
            | SolveOutcomeTruth::Infeasible
            | SolveOutcomeTruth::Cancelled => solver_failures += 1,
        }
    }

    let (
        event_count,
        timely_detections,
        missed_events,
        false_triggers,
        redundant_triggers,
        service_loss_mwh,
        audited_schedule_latency_steps,
    ) = match scenario.truth.material_event {
        None => (0, 0, 0, solver_invocations, 0, 0, None),
        Some(event) => {
            let qualifying: Vec<(u32, u32)> = audited_replans
                .iter()
                .copied()
                .filter(|(requested, completed)| {
                    *requested >= event.observable_at_step
                        && *requested <= event.response_deadline_step
                        && *completed <= event.response_deadline_step
                })
                .collect();
            let usable_at = qualifying.iter().map(|(_, completed)| *completed).min();
            let false_count = u32::try_from(
                replan_steps
                    .iter()
                    .filter(|step| {
                        **step < event.observable_at_step || **step > event.response_deadline_step
                    })
                    .count(),
            )
            .unwrap_or(u32::MAX);
            let detected = usable_at.is_some();
            let service_loss = if detected {
                event.service_loss_with_timely_replan_mwh
            } else {
                event.service_loss_without_replan_mwh
            };
            (
                1,
                if detected { 1 } else { 0 },
                if detected { 0 } else { 1 },
                false_count,
                u32::try_from(qualifying.len().saturating_sub(1)).unwrap_or(u32::MAX),
                service_loss,
                usable_at.map(|completed| completed - event.observable_at_step),
            )
        }
    };

    let total_penalty_mwh = service_loss_mwh.saturating_add(replanning_cost_mwh);
    PolicyRun {
        policy,
        scenario_id: scenario.id.clone(),
        actions,
        event_count,
        timely_detections,
        missed_events,
        false_triggers,
        redundant_triggers,
        solver_invocations,
        solver_completed,
        solver_failures,
        late_solver_completions,
        auditor_rejections,
        auditor_unavailable,
        accepted_replans,
        abstentions,
        service_loss_mwh,
        replanning_cost_mwh,
        total_penalty_mwh,
        audited_schedule_latency_steps,
    }
}

pub fn aggregate_runs(runs: &[PolicyRun]) -> AggregateMetrics {
    let mut metrics = AggregateMetrics {
        scenario_count: u32::try_from(runs.len()).unwrap_or(u32::MAX),
        ..AggregateMetrics::default()
    };

    for run in runs {
        metrics.event_count += run.event_count;
        metrics.timely_detections += run.timely_detections;
        metrics.missed_events += run.missed_events;
        metrics.false_triggers += run.false_triggers;
        metrics.redundant_triggers += run.redundant_triggers;
        metrics.solver_invocations += run.solver_invocations;
        metrics.solver_completed += run.solver_completed;
        metrics.solver_failures += run.solver_failures;
        metrics.late_solver_completions += run.late_solver_completions;
        metrics.auditor_rejections += run.auditor_rejections;
        metrics.auditor_unavailable += run.auditor_unavailable;
        metrics.accepted_replans += run.accepted_replans;
        metrics.abstentions += run.abstentions;
        metrics.service_loss_mwh += run.service_loss_mwh;
        metrics.replanning_cost_mwh += run.replanning_cost_mwh;
        metrics.total_penalty_mwh += run.total_penalty_mwh;
        if let Some(latency) = run.audited_schedule_latency_steps {
            metrics.audited_schedule_latency_step_sum += u64::from(latency);
            metrics.audited_schedule_latency_event_count += 1;
        }
    }

    metrics
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScenarioKind {
    Nominal,
    SlowDrift,
    SuddenOutage,
    NuisanceSpike,
    UncertaintyShock,
    TelemetryGap,
    HighValueDrift,
    HeavyTailedShock,
}

impl ScenarioKind {
    const ALL: [Self; 8] = [
        Self::Nominal,
        Self::SlowDrift,
        Self::SuddenOutage,
        Self::NuisanceSpike,
        Self::UncertaintyShock,
        Self::TelemetryGap,
        Self::HighValueDrift,
        Self::HeavyTailedShock,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::SlowDrift => "slow-drift",
            Self::SuddenOutage => "sudden-outage",
            Self::NuisanceSpike => "nuisance-spike",
            Self::UncertaintyShock => "uncertainty-shock",
            Self::TelemetryGap => "telemetry-gap",
            Self::HighValueDrift => "high-value-drift",
            Self::HeavyTailedShock => "heavy-tailed-shock",
        }
    }
}

fn observation(step: u32, variant: u32, telemetry_status: TelemetryStatus) -> DecisionObservation {
    DecisionObservation {
        step,
        telemetry_status,
        forecast_residual_kw: 500 + variant * 100,
        uncertainty_bound_kw: 1_000 + variant * 250,
        reserve_margin_mwh: 9_000 - variant * 200,
        estimated_avoidable_loss_mwh: 300 + variant * 20,
        estimated_solver_cost_mwh: 600,
        estimated_switching_cost_mwh: 200,
    }
}

fn build_scenario(kind: ScenarioKind, variant: u32) -> Scenario {
    let mut observations = Vec::with_capacity(8);
    for step in 0..8 {
        let mut item = observation(step, variant, TelemetryStatus::Fresh);
        match kind {
            ScenarioKind::Nominal => {}
            ScenarioKind::SlowDrift => {
                if step >= 2 {
                    item.forecast_residual_kw = 1_000 + (step - 2) * 2_000 + variant * 200;
                    item.uncertainty_bound_kw = 2_000 + (step - 2) * 1_000;
                    item.reserve_margin_mwh = 9_000 - (step - 2) * 1_000 - variant * 100;
                    if step >= 3 {
                        item.estimated_avoidable_loss_mwh = 3_500 + variant * 250;
                    }
                }
            }
            ScenarioKind::SuddenOutage => {
                if step >= 2 {
                    item.forecast_residual_kw = 12_000;
                    item.uncertainty_bound_kw = 10_000;
                    item.reserve_margin_mwh = 1_000 + variant * 100;
                    item.estimated_avoidable_loss_mwh = 12_000;
                }
            }
            ScenarioKind::NuisanceSpike => {
                if step == 3 {
                    item.forecast_residual_kw = 9_500;
                    item.uncertainty_bound_kw = 800;
                    item.reserve_margin_mwh = 9_000;
                    item.estimated_avoidable_loss_mwh = 100;
                }
            }
            ScenarioKind::UncertaintyShock => {
                if step >= 2 {
                    item.forecast_residual_kw = 1_000;
                    item.uncertainty_bound_kw = 9_000;
                    item.reserve_margin_mwh = 8_500;
                    item.estimated_avoidable_loss_mwh = 200;
                }
            }
            ScenarioKind::TelemetryGap => {
                if step == 2 {
                    item.telemetry_status = TelemetryStatus::Missing;
                } else if step == 3 {
                    item.telemetry_status = TelemetryStatus::Conflicting;
                } else if step >= 4 {
                    item.forecast_residual_kw = 9_000;
                    item.uncertainty_bound_kw = 7_000;
                    item.reserve_margin_mwh = 4_000;
                    item.estimated_avoidable_loss_mwh = 5_000;
                }
            }
            ScenarioKind::HighValueDrift => {
                if step >= 3 {
                    item.forecast_residual_kw = 3_000;
                    item.uncertainty_bound_kw = 3_500;
                    item.reserve_margin_mwh = 8_500;
                    item.estimated_avoidable_loss_mwh = 4_000 + variant * 250;
                }
            }
            ScenarioKind::HeavyTailedShock => {
                if step >= 4 {
                    item.forecast_residual_kw = 18_000 + variant * 500;
                    item.uncertainty_bound_kw = 12_000;
                    item.reserve_margin_mwh = 3_000;
                    item.estimated_avoidable_loss_mwh = 8_000;
                }
            }
        }
        observations.push(item);
    }

    let material_event = match kind {
        ScenarioKind::SlowDrift => Some(MaterialEvent {
            observable_at_step: 2,
            response_deadline_step: 5,
            service_loss_without_replan_mwh: 7_000 + u64::from(variant) * 500,
            service_loss_with_timely_replan_mwh: 2_000 + u64::from(variant) * 100,
        }),
        ScenarioKind::SuddenOutage => Some(MaterialEvent {
            observable_at_step: 2,
            response_deadline_step: 3,
            service_loss_without_replan_mwh: 14_000 + u64::from(variant) * 500,
            service_loss_with_timely_replan_mwh: 1_500 + u64::from(variant) * 100,
        }),
        ScenarioKind::TelemetryGap => Some(MaterialEvent {
            observable_at_step: 4,
            response_deadline_step: 6,
            service_loss_without_replan_mwh: 8_000 + u64::from(variant) * 300,
            service_loss_with_timely_replan_mwh: 2_500 + u64::from(variant) * 100,
        }),
        ScenarioKind::HighValueDrift => Some(MaterialEvent {
            observable_at_step: 3,
            response_deadline_step: 5,
            service_loss_without_replan_mwh: 6_500 + u64::from(variant) * 400,
            service_loss_with_timely_replan_mwh: 1_500 + u64::from(variant) * 100,
        }),
        ScenarioKind::HeavyTailedShock => Some(MaterialEvent {
            observable_at_step: 4,
            response_deadline_step: 5,
            service_loss_without_replan_mwh: 16_000 + u64::from(variant) * 600,
            service_loss_with_timely_replan_mwh: 2_000 + u64::from(variant) * 150,
        }),
        ScenarioKind::Nominal | ScenarioKind::NuisanceSpike | ScenarioKind::UncertaintyShock => {
            None
        }
    };

    let replan_executions = (0..8)
        .map(|requested_at_step| ReplanExecutionTruth {
            requested_at_step,
            solve_outcome: SolveOutcomeTruth::Completed {
                completed_at_step: requested_at_step + 1,
            },
            audit_outcome: AuditOutcomeTruth::Accepted,
        })
        .collect();

    Scenario {
        id: format!("{}-{:02}", kind.name(), variant),
        observations,
        truth: ScenarioTruth {
            material_event,
            replan_executions,
        },
    }
}

/// Build the deterministic 32-scenario synthetic panel (8 families x 4 variants).
pub fn frozen_scenarios() -> Vec<Scenario> {
    let mut scenarios = Vec::with_capacity(32);
    for kind in ScenarioKind::ALL {
        for variant in 0..4 {
            scenarios.push(build_scenario(kind, variant));
        }
    }
    scenarios
}

pub fn run_bakeoff() -> Vec<(PolicyKind, AggregateMetrics)> {
    let scenarios = frozen_scenarios();
    PolicyKind::ALL
        .into_iter()
        .map(|policy| {
            let runs: Vec<PolicyRun> = scenarios
                .iter()
                .map(|scenario| run_policy(policy, scenario))
                .collect();
            (policy, aggregate_runs(&runs))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario_of(kind: ScenarioKind, variant: u32) -> Scenario {
        build_scenario(kind, variant)
    }

    #[test]
    fn frozen_panel_has_32_deterministic_scenarios() {
        let first = frozen_scenarios();
        let second = frozen_scenarios();
        assert_eq!(first, second);
        assert_eq!(first.len(), 32);
    }

    #[test]
    fn invalid_telemetry_forces_abstention_for_every_policy() {
        for status in [
            TelemetryStatus::Stale,
            TelemetryStatus::Missing,
            TelemetryStatus::Conflicting,
        ] {
            let mut item = observation(3, 0, status);
            item.forecast_residual_kw = 99_000;
            item.uncertainty_bound_kw = 99_000;
            item.reserve_margin_mwh = 0;
            item.estimated_avoidable_loss_mwh = 99_000;
            for policy in PolicyKind::ALL {
                let scenario = Scenario {
                    id: "invalid-telemetry".to_owned(),
                    observations: vec![item.clone()],
                    truth: ScenarioTruth::default(),
                };
                let run = run_policy(policy, &scenario);
                assert_eq!(run.actions, vec![ReplanAction::Abstain]);
            }
        }
    }

    #[test]
    fn policies_cannot_observe_hidden_truth_for_identical_observations() {
        let scenario = scenario_of(ScenarioKind::SuddenOutage, 0);
        let mut counterfactual = scenario.clone();
        counterfactual.truth.material_event = Some(MaterialEvent {
            observable_at_step: 7,
            response_deadline_step: 7,
            service_loss_without_replan_mwh: 1,
            service_loss_with_timely_replan_mwh: 0,
        });

        for policy in PolicyKind::ALL {
            let original_actions = run_policy(policy, &scenario).actions;
            let counterfactual_actions = run_policy(policy, &counterfactual).actions;
            assert_eq!(original_actions, counterfactual_actions);
        }
    }

    #[test]
    fn hybrid_policy_triggers_on_hard_reserve_override() {
        let mut item = observation(1, 0, TelemetryStatus::Fresh);
        item.reserve_margin_mwh = 1_000;
        item.estimated_avoidable_loss_mwh = 0;
        assert_eq!(
            PolicyKind::HybridSafetyOverride.decide(&item),
            ReplanAction::RequestReplan
        );
    }

    #[test]
    fn decision_value_policy_suppresses_low_value_nuisance_spike() {
        let scenario = scenario_of(ScenarioKind::NuisanceSpike, 0);
        let run = run_policy(PolicyKind::DecisionValue, &scenario);
        assert_eq!(run.solver_invocations, 0);
        assert_eq!(run.false_triggers, 0);
    }

    #[test]
    fn residual_trigger_detects_nuisance_spike_but_evaluator_counts_it_as_false() {
        let scenario = scenario_of(ScenarioKind::NuisanceSpike, 0);
        let run = run_policy(PolicyKind::ForecastResidual, &scenario);
        assert!(run.solver_invocations > 0);
        assert_eq!(run.false_triggers, run.solver_invocations);
    }

    #[test]
    fn fixed_cadence_spends_solver_calls_on_nominal_scenarios() {
        let scenario = scenario_of(ScenarioKind::Nominal, 0);
        let run = run_policy(PolicyKind::FixedCadence, &scenario);
        assert!(run.solver_invocations > 0);
        assert_eq!(run.false_triggers, run.solver_invocations);
    }

    #[test]
    fn material_events_are_scored_only_against_the_evaluator_deadline() {
        let mut scenario = scenario_of(ScenarioKind::SuddenOutage, 0);
        scenario.truth.material_event = Some(MaterialEvent {
            observable_at_step: 2,
            response_deadline_step: 2,
            service_loss_without_replan_mwh: 5_000,
            service_loss_with_timely_replan_mwh: 1_000,
        });
        let run = run_policy(PolicyKind::FixedCadence, &scenario);
        assert_eq!(run.missed_events, 1);
        assert_eq!(run.timely_detections, 0);
        assert_eq!(run.service_loss_mwh, 5_000);
    }

    #[test]
    fn late_solver_completion_does_not_count_as_a_timely_replan() {
        let mut scenario = scenario_of(ScenarioKind::SuddenOutage, 0);
        let execution = scenario
            .truth
            .replan_executions
            .iter_mut()
            .find(|entry| entry.requested_at_step == 2)
            .unwrap();
        execution.solve_outcome = SolveOutcomeTruth::Completed {
            completed_at_step: 4,
        };

        let actions = scenario
            .observations
            .iter()
            .map(|observation| {
                if observation.step == 2 {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            })
            .collect();
        let run = evaluate_run(PolicyKind::HybridSafetyOverride, &scenario, actions);

        assert_eq!(run.solver_invocations, 1);
        assert_eq!(run.solver_completed, 1);
        assert_eq!(run.late_solver_completions, 1);
        assert_eq!(run.timely_detections, 0);
        assert_eq!(run.missed_events, 1);
    }

    #[test]
    fn an_independently_rejected_schedule_never_counts_as_usable() {
        let mut scenario = scenario_of(ScenarioKind::SuddenOutage, 0);
        let execution = scenario
            .truth
            .replan_executions
            .iter_mut()
            .find(|entry| entry.requested_at_step == 2)
            .unwrap();
        execution.audit_outcome = AuditOutcomeTruth::Rejected;

        let actions = scenario
            .observations
            .iter()
            .map(|observation| {
                if observation.step == 2 {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            })
            .collect();
        let run = evaluate_run(PolicyKind::HybridSafetyOverride, &scenario, actions);

        assert_eq!(run.solver_completed, 1);
        assert_eq!(run.auditor_rejections, 1);
        assert_eq!(run.accepted_replans, 0);
        assert_eq!(run.timely_detections, 0);
        assert_eq!(run.missed_events, 1);
    }

    #[test]
    fn a_solver_timeout_is_counted_but_never_treated_as_a_schedule() {
        let mut scenario = scenario_of(ScenarioKind::SuddenOutage, 0);
        let execution = scenario
            .truth
            .replan_executions
            .iter_mut()
            .find(|entry| entry.requested_at_step == 2)
            .unwrap();
        execution.solve_outcome = SolveOutcomeTruth::TimedOut;

        let actions = scenario
            .observations
            .iter()
            .map(|observation| {
                if observation.step == 2 {
                    ReplanAction::RequestReplan
                } else {
                    ReplanAction::KeepPlan
                }
            })
            .collect();
        let run = evaluate_run(PolicyKind::HybridSafetyOverride, &scenario, actions);

        assert_eq!(run.solver_invocations, 1);
        assert_eq!(run.solver_failures, 1);
        assert_eq!(run.accepted_replans, 0);
        assert_eq!(run.missed_events, 1);
    }

    #[test]
    fn every_policy_receives_the_same_time_local_observation_panel() {
        let scenarios = frozen_scenarios();
        let baseline: Vec<Vec<DecisionObservation>> = scenarios
            .iter()
            .map(|scenario| scenario.observations.clone())
            .collect();
        for policy in PolicyKind::ALL {
            let observed_again: Vec<Vec<DecisionObservation>> = scenarios
                .iter()
                .map(|scenario| {
                    let run = run_policy(policy, scenario);
                    assert_eq!(run.actions.len(), scenario.observations.len());
                    scenario.observations.clone()
                })
                .collect();
            assert_eq!(baseline, observed_again);
        }
    }

    #[test]
    fn all_material_events_are_counted_in_the_aggregate_denominator() {
        let scenarios = frozen_scenarios();
        let runs: Vec<PolicyRun> = scenarios
            .iter()
            .map(|scenario| run_policy(PolicyKind::HybridSafetyOverride, scenario))
            .collect();
        let aggregate = aggregate_runs(&runs);
        assert_eq!(aggregate.scenario_count, 32);
        assert_eq!(aggregate.event_count, 20);
        assert_eq!(aggregate.timely_detections + aggregate.missed_events, 20);
    }
}
