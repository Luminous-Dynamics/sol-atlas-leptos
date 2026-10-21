//! Bounded exact Pareto frontier for the v0.1 compute-energy reference model.
//!
//! Paradigm shift: keep cost, carbon, and facility-energy trade-offs explicit and
//! compare heuristic policies against the complete feasible allocation set for
//! small cases. This is not a production scheduler or a substitute for MILP/CP-SAT.
//! It requires homogeneous interruptible work, a full-horizon window, complete
//! objective attribution for every interval with residual capacity, and a strict
//! candidate cap. If the exact search exceeds its budget, no partial frontier is
//! returned or described as exact.

use crate::ai_compute_energy::{
    AssessmentStatus, Request, SchedulingPolicy, assess, schedule_deferrable,
};

/// Hard upper bound on enumerated complete schedules; the exact Pareto pass is O(n²).
pub const MAX_EXACT_FRONTIER_CANDIDATES: usize = 4_096;
/// Limit recursion depth and worst-case traversal for the small-instance oracle.
pub const MAX_EXACT_FRONTIER_INTERVALS: usize = 32;

/// Objective included in an exact frontier comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontierObjective {
    FacilityEnergy,
    ElectricityCost,
    GridCarbon,
}

/// Why a bounded exact frontier could not be produced.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExactFrontierError {
    InvalidCandidateLimit,
    HorizonTooLong {
        interval_count: usize,
        max_intervals: usize,
    },
    UnschedulableCapacity {
        status: AssessmentStatus,
    },
    MissingObjectiveAttribution {
        interval_id: String,
        objective: FrontierObjective,
    },
    NonFiniteDerivedObjective {
        interval_id: String,
    },
    CapacityOverflow,
    WorkUnitMagnitudeUnsupported,
    CandidateLimitExceeded {
        limit: usize,
    },
}

/// One non-dominated feasible allocation. Objective values cover deferrable work only.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ExactParetoSchedule {
    pub deferrable_work_units_by_interval: Vec<u64>,
    pub scheduled_work_units: u64,
    pub facility_energy_kwh: f64,
    pub electricity_cost_usd: f64,
    pub grid_carbon_gco2: f64,
}

/// Exact frontier over every feasible integer allocation evaluated under the bounded model.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ExactParetoFrontier {
    pub capacity_status: AssessmentStatus,
    /// Number of complete feasible allocations evaluated before Pareto filtering.
    pub evaluated_schedule_count: usize,
    /// Deterministically ordered; no returned option is dominated in all three objectives.
    pub schedules: Vec<ExactParetoSchedule>,
}

/// Enumerate all feasible allocations of the model's aggregate deferrable-work amount.
///
/// The result is exact only within `Request`'s v0.1 assumptions. It does not model
/// job-specific release/deadline windows, non-interruptible jobs, storage dispatch,
/// network power flow, thermal transients, task quality, or measured facility telemetry.
/// A budget overrun fails closed rather than returning a truncated list.
pub fn exact_pareto_frontier(
    request: &Request,
    max_candidates: usize,
) -> Result<ExactParetoFrontier, ExactFrontierError> {
    if max_candidates == 0 || max_candidates > MAX_EXACT_FRONTIER_CANDIDATES {
        return Err(ExactFrontierError::InvalidCandidateLimit);
    }
    if request.intervals.len() > MAX_EXACT_FRONTIER_INTERVALS {
        return Err(ExactFrontierError::HorizonTooLong {
            interval_count: request.intervals.len(),
            max_intervals: MAX_EXACT_FRONTIER_INTERVALS,
        });
    }

    let capacity = assess(request);
    if !matches!(
        capacity.status,
        AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited
    ) {
        return Err(ExactFrontierError::UnschedulableCapacity {
            status: capacity.status,
        });
    }
    let scheduled_work_units =
        capacity
            .max_deferrable_work_units
            .ok_or(ExactFrontierError::UnschedulableCapacity {
                status: capacity.status,
            })?;
    // Floating-point objective reporting cannot preserve every integer above 2^53.
    if scheduled_work_units > (1_u64 << 53) {
        return Err(ExactFrontierError::WorkUnitMagnitudeUnsupported);
    }

    let capacities = capacity
        .interval_assessments
        .iter()
        .map(|row| row.max_deferrable_work_units)
        .collect::<Vec<_>>();
    let intervals = &request.intervals;
    if capacities.len() != intervals.len() || intervals.is_empty() {
        return Err(ExactFrontierError::UnschedulableCapacity {
            status: AssessmentStatus::Invalid,
        });
    }

    let unit_energy_kwh = request
        .workloads
        .first()
        .map(|workload| workload.energy_kwh_per_work_unit)
        .ok_or(ExactFrontierError::UnschedulableCapacity {
            status: AssessmentStatus::Invalid,
        })?;

    // Validate all intervals with eligible residual capacity before enumeration.
    // A missing value is not treated as zero and cannot silently win the frontier.
    for (index, interval) in intervals.iter().enumerate() {
        if capacities[index] == 0 {
            continue;
        }
        let factor = interval.facility_energy_per_it_energy.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::FacilityEnergy,
            }
        })?;
        let price = interval.electricity_price_usd_per_kwh.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::ElectricityCost,
            }
        })?;
        let carbon = interval.grid_carbon_intensity_gco2_per_kwh.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::GridCarbon,
            }
        })?;
        let unit_facility_energy = factor * unit_energy_kwh;
        let unit_cost = unit_facility_energy * price;
        let unit_carbon = unit_facility_energy * carbon;
        if !unit_facility_energy.is_finite() || !unit_cost.is_finite() || !unit_carbon.is_finite() {
            return Err(ExactFrontierError::NonFiniteDerivedObjective {
                interval_id: interval.id.clone(),
            });
        }
    }

    // Suffix capacities restrict each branch to choices that can still complete
    // the required work amount, limiting dead-end traversal under the hard budget.
    let mut suffix_capacity = vec![0_u64; capacities.len() + 1];
    for index in (0..capacities.len()).rev() {
        suffix_capacity[index] = suffix_capacity[index + 1]
            .checked_add(capacities[index])
            .ok_or(ExactFrontierError::CapacityOverflow)?;
    }
    if suffix_capacity[0] < scheduled_work_units {
        return Err(ExactFrontierError::UnschedulableCapacity {
            status: capacity.status,
        });
    }

    let mut allocations = Vec::new();
    let mut current = Vec::with_capacity(capacities.len());
    enumerate_allocations(
        0,
        scheduled_work_units,
        &capacities,
        &suffix_capacity,
        &mut current,
        &mut allocations,
        max_candidates,
    )?;

    let mut evaluated = Vec::with_capacity(allocations.len());
    for allocation in allocations {
        evaluated.push(score_allocation(
            request,
            &allocation,
            scheduled_work_units,
        )?);
    }

    let mut schedules = Vec::new();
    for (index, candidate) in evaluated.iter().enumerate() {
        let dominated = evaluated.iter().enumerate().any(|(other_index, other)| {
            index != other_index && strictly_dominates(other, candidate)
        });
        if dominated {
            continue;
        }

        // Equal objective vectors have equivalent measured outcomes. Keep the
        // first allocation generated (lexicographically canonical) to avoid
        // publishing many schedule permutations as distinct Pareto choices.
        let duplicate_objective = evaluated
            .iter()
            .take(index)
            .any(|other| same_objectives(other, candidate));
        if !duplicate_objective {
            schedules.push(candidate.clone());
        }
    }

    schedules.sort_by(|left, right| {
        left.electricity_cost_usd
            .total_cmp(&right.electricity_cost_usd)
            .then_with(|| left.grid_carbon_gco2.total_cmp(&right.grid_carbon_gco2))
            .then_with(|| {
                left.facility_energy_kwh
                    .total_cmp(&right.facility_energy_kwh)
            })
            .then_with(|| {
                left.deferrable_work_units_by_interval
                    .cmp(&right.deferrable_work_units_by_interval)
            })
    });

    Ok(ExactParetoFrontier {
        capacity_status: capacity.status,
        evaluated_schedule_count: evaluated.len(),
        schedules,
    })
}

fn enumerate_allocations(
    index: usize,
    remaining: u64,
    capacities: &[u64],
    suffix_capacity: &[u64],
    current: &mut Vec<u64>,
    output: &mut Vec<Vec<u64>>,
    limit: usize,
) -> Result<(), ExactFrontierError> {
    if index == capacities.len() {
        if remaining != 0 {
            return Ok(());
        }
        if output.len() >= limit {
            return Err(ExactFrontierError::CandidateLimitExceeded { limit });
        }
        output.push(current.clone());
        return Ok(());
    }

    let minimum_here = remaining.saturating_sub(suffix_capacity[index + 1]);
    let maximum_here = capacities[index].min(remaining);
    if minimum_here > maximum_here {
        return Ok(());
    }

    for units in minimum_here..=maximum_here {
        current.push(units);
        enumerate_allocations(
            index + 1,
            remaining - units,
            capacities,
            suffix_capacity,
            current,
            output,
            limit,
        )?;
        current.pop();
    }
    Ok(())
}

fn score_allocation(
    request: &Request,
    allocation: &[u64],
    scheduled_work_units: u64,
) -> Result<ExactParetoSchedule, ExactFrontierError> {
    let unit_energy_kwh = request.workloads[0].energy_kwh_per_work_unit;
    let mut facility_energy_kwh = 0.0;
    let mut electricity_cost_usd = 0.0;
    let mut grid_carbon_gco2 = 0.0;

    for (index, units) in allocation.iter().copied().enumerate() {
        if units == 0 {
            continue;
        }
        let interval = &request.intervals[index];
        let factor = interval.facility_energy_per_it_energy.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::FacilityEnergy,
            }
        })?;
        let price = interval.electricity_price_usd_per_kwh.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::ElectricityCost,
            }
        })?;
        let carbon = interval.grid_carbon_intensity_gco2_per_kwh.ok_or_else(|| {
            ExactFrontierError::MissingObjectiveAttribution {
                interval_id: interval.id.clone(),
                objective: FrontierObjective::GridCarbon,
            }
        })?;

        let it_energy_kwh = (units as f64) * unit_energy_kwh;
        let interval_facility_energy_kwh = it_energy_kwh * factor;
        facility_energy_kwh += interval_facility_energy_kwh;
        electricity_cost_usd += interval_facility_energy_kwh * price;
        grid_carbon_gco2 += interval_facility_energy_kwh * carbon;
        if !facility_energy_kwh.is_finite()
            || !electricity_cost_usd.is_finite()
            || !grid_carbon_gco2.is_finite()
        {
            return Err(ExactFrontierError::NonFiniteDerivedObjective {
                interval_id: interval.id.clone(),
            });
        }
    }

    Ok(ExactParetoSchedule {
        deferrable_work_units_by_interval: allocation.to_vec(),
        scheduled_work_units,
        facility_energy_kwh,
        electricity_cost_usd,
        grid_carbon_gco2,
    })
}

fn strictly_dominates(left: &ExactParetoSchedule, right: &ExactParetoSchedule) -> bool {
    left.facility_energy_kwh <= right.facility_energy_kwh
        && left.electricity_cost_usd <= right.electricity_cost_usd
        && left.grid_carbon_gco2 <= right.grid_carbon_gco2
        && (left.facility_energy_kwh < right.facility_energy_kwh
            || left.electricity_cost_usd < right.electricity_cost_usd
            || left.grid_carbon_gco2 < right.grid_carbon_gco2)
}

fn same_objectives(left: &ExactParetoSchedule, right: &ExactParetoSchedule) -> bool {
    left.facility_energy_kwh
        .total_cmp(&right.facility_energy_kwh)
        .is_eq()
        && left
            .electricity_cost_usd
            .total_cmp(&right.electricity_cost_usd)
            .is_eq()
        && left
            .grid_carbon_gco2
            .total_cmp(&right.grid_carbon_gco2)
            .is_eq()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_compute_energy::WorkClass;

    fn fixture_request() -> Request {
        serde_json::from_str(include_str!(
            "../../schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json"
        ))
        .expect("checked-in synthetic scenario must parse")
    }

    fn strictly_dominates_test(left: &ExactParetoSchedule, right: &ExactParetoSchedule) -> bool {
        strictly_dominates(left, right)
    }

    #[test]
    fn exact_frontier_contains_only_nondominated_integer_schedules() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 10;
        request.intervals[0].electricity_price_usd_per_kwh = Some(0.02);
        request.intervals[1].electricity_price_usd_per_kwh = Some(0.30);
        request.intervals[2].electricity_price_usd_per_kwh = Some(0.40);
        request.intervals[3].electricity_price_usd_per_kwh = Some(0.50);
        assert_eq!(request.workloads[1].work_class, WorkClass::Deferrable);

        let frontier = exact_pareto_frontier(&request, 4096).expect("bounded exact search");
        assert!(
            frontier.schedules.len() >= 2,
            "price/carbon conflict should retain trade-offs"
        );
        assert!(frontier.evaluated_schedule_count > frontier.schedules.len());
        for candidate in &frontier.schedules {
            assert_eq!(candidate.scheduled_work_units, 10);
            assert_eq!(
                candidate
                    .deferrable_work_units_by_interval
                    .iter()
                    .sum::<u64>(),
                10
            );
            assert!(candidate.facility_energy_kwh.is_finite());
            assert!(candidate.electricity_cost_usd.is_finite());
            assert!(candidate.grid_carbon_gco2.is_finite());
        }
        for (index, candidate) in frontier.schedules.iter().enumerate() {
            for (other_index, other) in frontier.schedules.iter().enumerate() {
                if index != other_index {
                    assert!(
                        !strictly_dominates_test(other, candidate),
                        "frontier contains a dominated schedule: {candidate:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn exact_frontier_is_deterministic_and_covers_reference_policies() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 10;
        request.intervals[0].electricity_price_usd_per_kwh = Some(0.02);
        request.intervals[1].electricity_price_usd_per_kwh = Some(0.30);
        request.intervals[2].electricity_price_usd_per_kwh = Some(0.40);
        request.intervals[3].electricity_price_usd_per_kwh = Some(0.50);

        let frontier = exact_pareto_frontier(&request, 4096).expect("bounded exact search");
        assert_eq!(
            frontier,
            exact_pareto_frontier(&request, 4096).expect("repeat must match")
        );

        for policy in [
            SchedulingPolicy::EarliestReady,
            SchedulingPolicy::LowestCost,
            SchedulingPolicy::LowestCarbon,
        ] {
            let baseline = schedule_deferrable(&request, policy);
            let cost = baseline
                .modelled_electricity_cost_usd
                .expect("fixture declares prices");
            let carbon = baseline
                .modelled_grid_carbon_gco2
                .expect("fixture declares carbon attribution");
            let facility_energy = baseline
                .deferrable_facility_energy_kwh
                .expect("fixture declares facility factors");
            assert!(
                frontier.schedules.iter().any(|candidate| {
                    candidate.facility_energy_kwh <= facility_energy
                        && candidate.electricity_cost_usd <= cost
                        && candidate.grid_carbon_gco2 <= carbon
                }),
                "exact frontier must weakly dominate or match policy {policy:?}"
            );
        }
    }

    #[test]
    fn exact_frontier_fails_closed_when_candidate_budget_is_exceeded() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 10;
        assert_eq!(
            exact_pareto_frontier(&request, 1),
            Err(ExactFrontierError::CandidateLimitExceeded { limit: 1 })
        );
    }

    #[test]
    fn exact_frontier_requires_complete_objective_attribution() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 10;
        request.intervals[0].grid_carbon_intensity_gco2_per_kwh = None;
        assert_eq!(
            exact_pareto_frontier(&request, 4096),
            Err(ExactFrontierError::MissingObjectiveAttribution {
                interval_id: "t0".to_owned(),
                objective: FrontierObjective::GridCarbon,
            })
        );
    }

    #[test]
    fn exact_frontier_rejects_horizons_above_the_recursion_bound() {
        let mut request = fixture_request();
        let template = request.intervals[0].clone();
        request.intervals = (0..=MAX_EXACT_FRONTIER_INTERVALS)
            .map(|index| {
                let mut interval = template.clone();
                interval.id = format!("t{index}");
                interval
            })
            .collect();
        assert_eq!(
            exact_pareto_frontier(&request, 4096),
            Err(ExactFrontierError::HorizonTooLong {
                interval_count: MAX_EXACT_FRONTIER_INTERVALS + 1,
                max_intervals: MAX_EXACT_FRONTIER_INTERVALS,
            })
        );
    }

    #[test]
    fn invalid_budgets_are_rejected() {
        let request = fixture_request();
        assert_eq!(
            exact_pareto_frontier(&request, 0),
            Err(ExactFrontierError::InvalidCandidateLimit)
        );
        assert_eq!(
            exact_pareto_frontier(&request, MAX_EXACT_FRONTIER_CANDIDATES + 1),
            Err(ExactFrontierError::InvalidCandidateLimit)
        );
    }
}
