use energy_temporal_mesh_voi::{frozen_scenarios, plan_mesh, MeshStrategy};

fn main() {
    println!("# schema=energy-temporal-mesh-budget-report-v0.1");
    println!(
        "scenario_id,strategy,event_count,compute_budget_units,compute_cost_units,budget_remaining_units,safety_resolution_violations,unresolved_loss_units,accepted,refined_event_ids"
    );

    for scenario in frozen_scenarios() {
        for strategy in MeshStrategy::ALL {
            match plan_mesh(
                &scenario.events,
                strategy,
                scenario.compute_budget_units,
            ) {
                Ok(plan) => println!(
                    "{},{},{},{},{},{},{},{},{},{}",
                    scenario.id,
                    strategy.name(),
                    scenario.events.len(),
                    scenario.compute_budget_units,
                    plan.compute_cost_units,
                    plan.budget_remaining_units,
                    plan.safety_resolution_violations,
                    plan.unresolved_loss_units,
                    plan.accepted(),
                    plan.refined_event_ids.join("|")
                ),
                Err(error) => println!(
                    "{},{},{},{},NA,NA,NA,NA,false,error:{}",
                    scenario.id,
                    strategy.name(),
                    scenario.events.len(),
                    scenario.compute_budget_units,
                    error
                ),
            }
        }
    }
}
