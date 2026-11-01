use energy_replan_policy_bakeoff::run_bakeoff;

fn main() {
    println!("# schema=energy-replan-policy-report-v0.1");
    println!(
        "policy,scenario_count,event_count,timely_detections,missed_events,false_triggers,redundant_triggers,solver_invocations,solver_completed,solver_failures,late_solver_completions,auditor_rejections,auditor_unavailable,accepted_replans,abstentions,service_loss_units,replanning_cost_units,total_penalty_units,mean_audited_schedule_latency_steps"
    );

    for (policy, metrics) in run_bakeoff() {
        let mean_latency = metrics
            .mean_audited_schedule_latency_steps()
            .map(|value| format!("{value:.3}"))
            .unwrap_or_else(|| "NA".to_owned());
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            policy.name(),
            metrics.scenario_count,
            metrics.event_count,
            metrics.timely_detections,
            metrics.missed_events,
            metrics.false_triggers,
            metrics.redundant_triggers,
            metrics.solver_invocations,
            metrics.solver_completed,
            metrics.solver_failures,
            metrics.late_solver_completions,
            metrics.auditor_rejections,
            metrics.auditor_unavailable,
            metrics.accepted_replans,
            metrics.abstentions,
            metrics.service_loss_units,
            metrics.replanning_cost_units,
            metrics.total_penalty_units,
            mean_latency
        );
    }
}
