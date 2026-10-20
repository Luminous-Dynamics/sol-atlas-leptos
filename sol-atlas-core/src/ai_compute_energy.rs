//! Small compute-energy reference evaluator without an external optimization solver.
//!
//! This v0.1 evaluator is deliberately narrower than a production scheduler:
//! it supports one site, a finite aligned time grid, hard per-interval protected
//! floors, and homogeneous work units whose release/deadline window spans the
//! horizon. It checks first-order grid, thermal and direct-water energy bounds.
//! It does not model a power-flow network, cooling-fluid dynamics, transient
//! server behavior, task-specific quality, storage dispatch or a real facility.
//!
//! The interval facility-energy multiplier is a synthetic model coefficient,
//! not standardized PUE. Production PUE requires measured facility and IT energy
//! over a defined measurement period.

/// A time interval. Missing required resource constraints must remain unknown.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub id: String,
    pub duration_hours: f64,
    pub grid_import_limit_kw: Option<f64>,
    pub facility_energy_per_it_energy: Option<f64>,
    pub thermal_it_power_limit_kw: Option<f64>,
    pub cooling_water_limit_l: Option<f64>,
    pub water_l_per_it_kwh: Option<f64>,
    /// Optional attributed electricity tariff. Negative wholesale prices are valid.
    #[serde(default)]
    pub electricity_price_usd_per_kwh: Option<f64>,
    /// Optional attributed interval carbon intensity.
    #[serde(default)]
    pub grid_carbon_intensity_gco2_per_kwh: Option<f64>,
    /// Context only; the thermal cap is supplied explicitly and is not inferred here.
    #[serde(default)]
    pub ambient_temperature_c: Option<f64>,
    /// Scenario annotations only; hard outages must also be reflected in resource limits.
    #[serde(default)]
    pub failure_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub site_id: String,
    pub it_power_capacity_kw: f64,
    #[serde(default)]
    pub region_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkClass {
    Protected,
    Deferrable,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    pub workload_id: String,
    pub work_class: WorkClass,
    pub required_work_units: u64,
    pub energy_kwh_per_work_unit: f64,
    pub release_interval_index: usize,
    pub deadline_interval_index: usize,
    /// Hard lower bound for each time interval. Protected work must equal its
    /// required units in total for this v0.1 contract.
    pub minimum_accepted_units_by_interval: Vec<u64>,
    pub service_quality_profile_id: String,
    pub interruptible: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageProfile {
    pub capacity_kwh: f64,
    pub opening_stock_kwh: f64,
    pub charge_limit_kw: f64,
    pub discharge_limit_kw: f64,
    pub round_trip_efficiency: f64,
    pub terminal_reserve_kwh: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleExpectations {
    pub protected_work_units: u64,
    pub max_deferrable_work_units: u64,
    pub resource_limits_hold: bool,
}

/// A typed projection of the v0.1 scenario envelope. The root and nested
/// records reject unknown keys so a future physical constraint cannot silently
/// disappear during deserialization.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub fixture_id: String,
    pub synthetic: bool,
    pub claim_ceiling: String,
    pub site: Site,
    pub intervals: Vec<Interval>,
    pub workloads: Vec<Workload>,
    pub baseline_policy: String,
    pub candidate_policies: Vec<String>,
    #[serde(default)]
    pub storage: Option<StorageProfile>,
    #[serde(default)]
    pub oracle_expectations: Option<OracleExpectations>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    /// All demand can fit in the first-order resource envelope.
    Feasible,
    /// Protected floors fit, but requested deferrable work exceeds residual capacity.
    CapacityLimited,
    /// A protected-service floor cannot fit.
    Infeasible,
    /// A required resource bound is unknown; no numeric plan is returned.
    Indeterminate,
    /// The request is structurally valid but outside this small evaluator's scope.
    Unsupported,
    /// Malformed values, identities, or interval alignment.
    Invalid,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct IntervalAssessment {
    pub interval_id: String,
    pub effective_it_power_cap_kw: f64,
    pub it_energy_capacity_kwh: f64,
    pub protected_floor_energy_kwh: f64,
    pub residual_energy_for_deferrable_kwh: f64,
    pub max_deferrable_work_units: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Assessment {
    pub status: AssessmentStatus,
    pub interval_assessments: Vec<IntervalAssessment>,
    pub protected_floor_work_units: Option<u64>,
    pub protected_floor_met: Option<bool>,
    pub requested_deferrable_work_units: Option<u64>,
    pub max_deferrable_work_units: Option<u64>,
    /// Sorted, deduplicated machine-readable-ish reasons for the disposition.
    pub blockers: Vec<String>,
}

impl Assessment {
    fn empty(status: AssessmentStatus, mut blockers: Vec<String>) -> Self {
        blockers.sort();
        blockers.dedup();
        Self {
            status,
            interval_assessments: Vec::new(),
            protected_floor_work_units: None,
            protected_floor_met: None,
            requested_deferrable_work_units: None,
            max_deferrable_work_units: None,
            blockers,
        }
    }
}

const NUMERIC_TOLERANCE: f64 = 1.0e-9;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn checked_sum(values: impl IntoIterator<Item = u64>) -> Option<u64> {
    values
        .into_iter()
        .try_fold(0_u64, |sum, value| sum.checked_add(value))
}

/// Assess whether protected floors fit and determine the maximum residual
/// deferrable work for the declared homogeneous-work-unit scope.
///
/// This function is a reference capacity bound, not a production scheduler.
/// It does not emit a per-workload dispatch schedule. The returned aggregate
/// is valid only when all workloads share one energy-per-work-unit coefficient
/// and all deferrable workloads can run anywhere within the full horizon.
pub fn assess(request: &Request) -> Assessment {
    let mut invalid = Vec::new();
    let mut missing = Vec::new();
    let mut unsupported = Vec::new();

    if request.schema_version != "0.1" {
        unsupported.push("scenario_schema_version_unsupported".to_owned());
    }
    if request.fixture_id.trim().is_empty() {
        invalid.push("fixture_id_empty".to_owned());
    }
    if request.claim_ceiling.trim().is_empty() {
        invalid.push("claim_ceiling_empty".to_owned());
    }
    if request.baseline_policy != "fixed_earliest_ready" {
        unsupported.push("baseline_policy_unsupported_v0_1".to_owned());
    }
    if request.candidate_policies.is_empty() {
        invalid.push("candidate_policies_empty".to_owned());
    }
    if request.storage.is_some() {
        unsupported.push("storage_dispatch_unsupported_v0_1".to_owned());
    }

    if request.site.site_id.trim().is_empty() {
        invalid.push("site_id_empty".to_owned());
    }
    if !request.site.it_power_capacity_kw.is_finite() || request.site.it_power_capacity_kw <= 0.0 {
        invalid.push("site_it_power_capacity_not_positive_finite".to_owned());
    }
    if request.intervals.is_empty() {
        invalid.push("intervals_empty".to_owned());
    }
    if request.workloads.is_empty() {
        invalid.push("workloads_empty".to_owned());
    }

    let mut interval_ids = std::collections::BTreeSet::new();
    for (index, interval) in request.intervals.iter().enumerate() {
        if interval.id.trim().is_empty() {
            invalid.push(format!("interval_{index}_id_empty"));
        }
        if !interval_ids.insert(interval.id.as_str()) {
            invalid.push(format!("interval_id_duplicate:{}", interval.id));
        }
        if !interval.duration_hours.is_finite() || interval.duration_hours <= 0.0 {
            invalid.push(format!("interval_{}_duration_invalid", interval.id));
        }

        if interval.grid_import_limit_kw.is_none() {
            missing.push(format!(
                "interval_{}_grid_import_limit_unknown",
                interval.id
            ));
        }
        if interval.facility_energy_per_it_energy.is_none() {
            missing.push(format!(
                "interval_{}_facility_energy_factor_unknown",
                interval.id
            ));
        }
        if interval.thermal_it_power_limit_kw.is_none() {
            missing.push(format!(
                "interval_{}_thermal_it_power_limit_unknown",
                interval.id
            ));
        }
        if interval.cooling_water_limit_l.is_none() {
            missing.push(format!(
                "interval_{}_cooling_water_limit_unknown",
                interval.id
            ));
        }
        if interval.water_l_per_it_kwh.is_none() {
            missing.push(format!("interval_{}_water_intensity_unknown", interval.id));
        }

        for (name, value) in [
            ("grid_import_limit_kw", interval.grid_import_limit_kw),
            (
                "facility_energy_per_it_energy",
                interval.facility_energy_per_it_energy,
            ),
            (
                "thermal_it_power_limit_kw",
                interval.thermal_it_power_limit_kw,
            ),
            ("cooling_water_limit_l", interval.cooling_water_limit_l),
            ("water_l_per_it_kwh", interval.water_l_per_it_kwh),
            (
                "grid_carbon_intensity_gco2_per_kwh",
                interval.grid_carbon_intensity_gco2_per_kwh,
            ),
        ] {
            if let Some(value) = value {
                if !finite_nonnegative(value) {
                    invalid.push(format!("interval_{}_{}_invalid", interval.id, name));
                }
            }
        }
        // Wholesale power prices may legitimately be negative during some
        // market intervals. Require finiteness, but do not assume a nonnegative
        // price floor in the economic model.
        if let Some(price) = interval.electricity_price_usd_per_kwh {
            if !price.is_finite() {
                invalid.push(format!(
                    "interval_{}_electricity_price_usd_per_kwh_invalid",
                    interval.id
                ));
            }
        }
        if let Some(value) = interval.facility_energy_per_it_energy {
            if value < 1.0 {
                invalid.push(format!(
                    "interval_{}_facility_energy_factor_below_one",
                    interval.id
                ));
            }
        }
    }

    let interval_count = request.intervals.len();
    let horizon_last = interval_count.saturating_sub(1);
    let mut workload_ids = std::collections::BTreeSet::new();
    let mut common_energy: Option<f64> = None;
    let mut protected_units = Vec::new();
    let mut deferrable_units = Vec::new();
    let mut deferrable_quality_profile: Option<&str> = None;

    for workload in &request.workloads {
        if workload.workload_id.trim().is_empty() {
            invalid.push("workload_id_empty".to_owned());
        }
        if workload.service_quality_profile_id.trim().is_empty() {
            invalid.push(format!(
                "workload_{}_quality_profile_empty",
                workload.workload_id
            ));
        }
        if !workload_ids.insert(workload.workload_id.as_str()) {
            invalid.push(format!("workload_id_duplicate:{}", workload.workload_id));
        }
        if !workload.energy_kwh_per_work_unit.is_finite()
            || workload.energy_kwh_per_work_unit <= 0.0
        {
            invalid.push(format!(
                "workload_{}_energy_per_unit_invalid",
                workload.workload_id
            ));
        }
        if workload.minimum_accepted_units_by_interval.len() != interval_count {
            invalid.push(format!(
                "workload_{}_floor_length_mismatch",
                workload.workload_id
            ));
            continue;
        }
        if workload.release_interval_index > workload.deadline_interval_index
            || workload.deadline_interval_index >= interval_count
        {
            invalid.push(format!("workload_{}_window_invalid", workload.workload_id));
        }
        for (index, floor) in workload
            .minimum_accepted_units_by_interval
            .iter()
            .copied()
            .enumerate()
        {
            if (index < workload.release_interval_index || index > workload.deadline_interval_index)
                && floor > 0
            {
                invalid.push(format!(
                    "workload_{}_floor_outside_window",
                    workload.workload_id
                ));
            }
        }

        let Some(floor_sum) =
            checked_sum(workload.minimum_accepted_units_by_interval.iter().copied())
        else {
            invalid.push(format!(
                "workload_{}_floor_sum_overflow",
                workload.workload_id
            ));
            continue;
        };
        if floor_sum > workload.required_work_units {
            invalid.push(format!(
                "workload_{}_floor_exceeds_demand",
                workload.workload_id
            ));
        }

        match workload.work_class {
            WorkClass::Protected => {
                if floor_sum != workload.required_work_units {
                    unsupported.push(format!(
                        "workload_{}_protected_demand_must_equal_floors_v0_1",
                        workload.workload_id
                    ));
                }
                protected_units.push(workload.required_work_units);
            }
            WorkClass::Deferrable => {
                if !workload.interruptible {
                    unsupported.push(format!(
                        "workload_{}_deferrable_not_interruptible_v0_1",
                        workload.workload_id
                    ));
                }
                if let Some(profile) = deferrable_quality_profile {
                    if profile != workload.service_quality_profile_id.as_str() {
                        unsupported.push(
                            "heterogeneous_deferrable_quality_profiles_unsupported_v0_1".to_owned(),
                        );
                    }
                } else {
                    deferrable_quality_profile = Some(&workload.service_quality_profile_id);
                }
                if floor_sum != 0 {
                    unsupported.push(format!(
                        "workload_{}_deferrable_floors_unsupported_v0_1",
                        workload.workload_id
                    ));
                }
                if workload.release_interval_index != 0
                    || workload.deadline_interval_index != horizon_last
                {
                    unsupported.push(format!(
                        "workload_{}_must_span_full_horizon_v0_1",
                        workload.workload_id
                    ));
                }
                deferrable_units.push(workload.required_work_units);
            }
        }

        if let Some(existing) = common_energy {
            if (existing - workload.energy_kwh_per_work_unit).abs()
                > NUMERIC_TOLERANCE.max(existing.abs() * 1.0e-9)
            {
                unsupported
                    .push("heterogeneous_work_energy_coefficients_unsupported_v0_1".to_owned());
            }
        } else {
            common_energy = Some(workload.energy_kwh_per_work_unit);
        }
    }

    if !invalid.is_empty() {
        return Assessment::empty(AssessmentStatus::Invalid, invalid);
    }
    if !missing.is_empty() {
        return Assessment::empty(AssessmentStatus::Indeterminate, missing);
    }
    if !unsupported.is_empty() {
        return Assessment::empty(AssessmentStatus::Unsupported, unsupported);
    }

    let Some(protected_floor_work_units) = checked_sum(protected_units) else {
        return Assessment::empty(
            AssessmentStatus::Invalid,
            vec!["protected_work_sum_overflow".to_owned()],
        );
    };
    let Some(requested_deferrable_work_units) = checked_sum(deferrable_units) else {
        return Assessment::empty(
            AssessmentStatus::Invalid,
            vec!["deferrable_work_sum_overflow".to_owned()],
        );
    };
    let Some(unit_energy) = common_energy else {
        return Assessment::empty(
            AssessmentStatus::Invalid,
            vec!["common_energy_coefficient_missing".to_owned()],
        );
    };

    let mut interval_assessments = Vec::with_capacity(interval_count);
    let mut total_deferrable_ceiling = 0_u64;
    let mut floor_infeasible = false;
    let mut blockers = Vec::new();

    for (index, interval) in request.intervals.iter().enumerate() {
        // Missing inputs were rejected above, so these unwraps are guarded by
        // the explicit missing-input gate and cannot turn Unknown into zero.
        let grid_kw = interval
            .grid_import_limit_kw
            .expect("missing checked above");
        let facility_factor = interval
            .facility_energy_per_it_energy
            .expect("missing checked above");
        let thermal_kw = interval
            .thermal_it_power_limit_kw
            .expect("missing checked above");
        let water_limit_l = interval
            .cooling_water_limit_l
            .expect("missing checked above");
        let water_l_per_kwh = interval.water_l_per_it_kwh.expect("missing checked above");

        let grid_limited_it_kw = grid_kw / facility_factor;
        let water_limited_it_kw = if water_l_per_kwh == 0.0 {
            f64::INFINITY
        } else {
            water_limit_l / water_l_per_kwh / interval.duration_hours
        };
        let effective_kw = request
            .site
            .it_power_capacity_kw
            .min(thermal_kw)
            .min(grid_limited_it_kw)
            .min(water_limited_it_kw);
        let it_energy_kwh = effective_kw * interval.duration_hours;

        if !effective_kw.is_finite()
            || !it_energy_kwh.is_finite()
            || it_energy_kwh > u64::MAX as f64
        {
            return Assessment::empty(
                AssessmentStatus::Invalid,
                vec![format!("interval_{}_derived_capacity_invalid", interval.id)],
            );
        }

        let floor_energy_kwh = request
            .workloads
            .iter()
            .map(|workload| {
                workload.minimum_accepted_units_by_interval[index] as f64
                    * workload.energy_kwh_per_work_unit
            })
            .sum::<f64>();
        if !floor_energy_kwh.is_finite() {
            return Assessment::empty(
                AssessmentStatus::Invalid,
                vec![format!("interval_{}_floor_energy_invalid", interval.id)],
            );
        }

        let residual_kwh = it_energy_kwh - floor_energy_kwh;
        if residual_kwh < -NUMERIC_TOLERANCE {
            floor_infeasible = true;
            blockers.push(format!(
                "interval_{}_protected_floor_exceeds_capacity",
                interval.id
            ));
        }
        let safe_residual_kwh = residual_kwh.max(0.0);
        let units_float = ((safe_residual_kwh + NUMERIC_TOLERANCE) / unit_energy).floor();
        if !units_float.is_finite() || units_float > u64::MAX as f64 {
            return Assessment::empty(
                AssessmentStatus::Invalid,
                vec![format!(
                    "interval_{}_derived_work_capacity_invalid",
                    interval.id
                )],
            );
        }
        let units = units_float as u64;
        total_deferrable_ceiling = match total_deferrable_ceiling.checked_add(units) {
            Some(sum) => sum,
            None => {
                return Assessment::empty(
                    AssessmentStatus::Invalid,
                    vec!["deferrable_capacity_sum_overflow".to_owned()],
                );
            }
        };

        interval_assessments.push(IntervalAssessment {
            interval_id: interval.id.clone(),
            effective_it_power_cap_kw: effective_kw,
            it_energy_capacity_kwh: it_energy_kwh,
            protected_floor_energy_kwh: floor_energy_kwh,
            residual_energy_for_deferrable_kwh: safe_residual_kwh,
            max_deferrable_work_units: units,
        });
    }

    if floor_infeasible {
        blockers.sort();
        blockers.dedup();
        return Assessment {
            status: AssessmentStatus::Infeasible,
            interval_assessments,
            protected_floor_work_units: Some(protected_floor_work_units),
            protected_floor_met: Some(false),
            requested_deferrable_work_units: Some(requested_deferrable_work_units),
            max_deferrable_work_units: None,
            blockers,
        };
    }

    let max_deferrable_work_units = total_deferrable_ceiling.min(requested_deferrable_work_units);
    let status = if max_deferrable_work_units == requested_deferrable_work_units {
        AssessmentStatus::Feasible
    } else {
        blockers.push("deferrable_demand_exceeds_residual_capacity".to_owned());
        AssessmentStatus::CapacityLimited
    };
    blockers.sort();
    blockers.dedup();

    Assessment {
        status,
        interval_assessments,
        protected_floor_work_units: Some(protected_floor_work_units),
        protected_floor_met: Some(true),
        requested_deferrable_work_units: Some(requested_deferrable_work_units),
        max_deferrable_work_units: Some(max_deferrable_work_units),
        blockers,
    }
}

/// Scheduling objective for homogeneous, fully horizon-flexible deferrable work.
///
/// This transparent baseline ladder holds first-order interval capacity and
/// protected floors fixed, changing only the order in which eligible intervals
/// receive deferrable work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicy {
    /// Fill earlier intervals first, independent of tariff or carbon factors.
    EarliestReady,
    /// Fill lowest modelled facility electricity-cost intervals first.
    LowestCost,
    /// Fill lowest modelled grid-carbon-intensity intervals first.
    LowestCarbon,
}

/// Result of scheduling aggregate homogeneous deferrable work.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct DeferrableScheduleAssessment {
    pub status: AssessmentStatus,
    pub policy: SchedulingPolicy,
    pub protected_floor_work_units_by_interval: Option<Vec<u64>>,
    pub deferrable_work_units_by_interval: Option<Vec<u64>>,
    pub scheduled_work_units: Option<u64>,
    pub deferrable_it_energy_kwh: Option<f64>,
    pub deferrable_facility_energy_kwh: Option<f64>,
    /// None means an interval receiving work lacks an attributed price.
    pub modelled_electricity_cost_usd: Option<f64>,
    /// None means an interval receiving work lacks attributed carbon data.
    pub modelled_grid_carbon_gco2: Option<f64>,
    pub blockers: Vec<String>,
}

/// Comparison of the reference policy set without collapsing cost and carbon
/// into a single weighted score. The frontier is withheld when candidates
/// cannot be compared under complete, attributed objective data.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct PolicyFrontierAssessment {
    pub status: AssessmentStatus,
    pub policy_assessments: Vec<DeferrableScheduleAssessment>,
    pub non_dominated_policies: Vec<SchedulingPolicy>,
    pub blockers: Vec<String>,
}

impl DeferrableScheduleAssessment {
    fn no_schedule(
        status: AssessmentStatus,
        policy: SchedulingPolicy,
        mut blockers: Vec<String>,
    ) -> Self {
        blockers.sort();
        blockers.dedup();
        Self {
            status,
            policy,
            protected_floor_work_units_by_interval: None,
            deferrable_work_units_by_interval: None,
            scheduled_work_units: None,
            deferrable_it_energy_kwh: None,
            deferrable_facility_energy_kwh: None,
            modelled_electricity_cost_usd: None,
            modelled_grid_carbon_gco2: None,
            blockers,
        }
    }
}

/// Schedule aggregate deferrable work using an explicit objective. LowestCost
/// and LowestCarbon fail closed when objective data is missing for any interval
/// with eligible capacity. This is a bounded baseline, not a production
/// cluster scheduler: all deferrable jobs must share one work-energy coefficient
/// and span the full time horizon, as enforced by `assess()`.
pub fn schedule_deferrable(
    request: &Request,
    policy: SchedulingPolicy,
) -> DeferrableScheduleAssessment {
    let capacity = assess(request);
    if !matches!(
        capacity.status,
        AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited
    ) {
        return DeferrableScheduleAssessment::no_schedule(
            capacity.status,
            policy,
            capacity.blockers,
        );
    }

    let intervals = &request.intervals;
    let per_interval_capacity = capacity
        .interval_assessments
        .iter()
        .map(|row| row.max_deferrable_work_units)
        .collect::<Vec<_>>();

    let mut blockers = Vec::new();
    match policy {
        SchedulingPolicy::EarliestReady => {}
        SchedulingPolicy::LowestCost => {
            for (index, interval) in intervals.iter().enumerate() {
                if per_interval_capacity[index] > 0
                    && interval.electricity_price_usd_per_kwh.is_none()
                {
                    blockers.push(format!(
                        "interval_{}_electricity_price_unknown",
                        interval.id
                    ));
                }
            }
        }
        SchedulingPolicy::LowestCarbon => {
            for (index, interval) in intervals.iter().enumerate() {
                if per_interval_capacity[index] > 0
                    && interval.grid_carbon_intensity_gco2_per_kwh.is_none()
                {
                    blockers.push(format!(
                        "interval_{}_grid_carbon_intensity_unknown",
                        interval.id
                    ));
                }
            }
        }
    }
    if !blockers.is_empty() {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Indeterminate,
            policy,
            blockers,
        );
    }

    // Inputs can each be finite while their derived per-work-unit objective
    // overflows. Reject those cases before sorting; otherwise an infinite rank
    // can silently influence ordering even when that interval receives no work.
    let unit_energy_for_objective = request.workloads[0].energy_kwh_per_work_unit;
    for (index, interval) in intervals.iter().enumerate() {
        if per_interval_capacity[index] == 0 {
            continue;
        }
        let facility_factor = interval
            .facility_energy_per_it_energy
            .expect("a schedulable assessment requires known facility energy factors");
        let objective_per_unit = match policy {
            SchedulingPolicy::EarliestReady => 0.0,
            SchedulingPolicy::LowestCost => {
                interval
                    .electricity_price_usd_per_kwh
                    .expect("eligible cost intervals were checked above")
                    * facility_factor
                    * unit_energy_for_objective
            }
            SchedulingPolicy::LowestCarbon => {
                interval
                    .grid_carbon_intensity_gco2_per_kwh
                    .expect("eligible carbon intervals were checked above")
                    * facility_factor
                    * unit_energy_for_objective
            }
        };
        if !objective_per_unit.is_finite() {
            return DeferrableScheduleAssessment::no_schedule(
                AssessmentStatus::Invalid,
                policy,
                vec![format!(
                    "interval_{}_derived_objective_per_unit_non_finite",
                    interval.id
                )],
            );
        }
    }

    let mut order = (0..intervals.len()).collect::<Vec<_>>();
    match policy {
        SchedulingPolicy::EarliestReady => {}
        SchedulingPolicy::LowestCost => order.sort_by(|left, right| {
            let left_score = intervals[*left]
                .electricity_price_usd_per_kwh
                .unwrap_or(0.0)
                * intervals[*left]
                    .facility_energy_per_it_energy
                    .unwrap_or(1.0);
            let right_score = intervals[*right]
                .electricity_price_usd_per_kwh
                .unwrap_or(0.0)
                * intervals[*right]
                    .facility_energy_per_it_energy
                    .unwrap_or(1.0);
            left_score
                .total_cmp(&right_score)
                .then_with(|| left.cmp(right))
        }),
        SchedulingPolicy::LowestCarbon => order.sort_by(|left, right| {
            let left_score = intervals[*left]
                .grid_carbon_intensity_gco2_per_kwh
                .unwrap_or(0.0)
                * intervals[*left]
                    .facility_energy_per_it_energy
                    .unwrap_or(1.0);
            let right_score = intervals[*right]
                .grid_carbon_intensity_gco2_per_kwh
                .unwrap_or(0.0)
                * intervals[*right]
                    .facility_energy_per_it_energy
                    .unwrap_or(1.0);
            left_score
                .total_cmp(&right_score)
                .then_with(|| left.cmp(right))
        }),
    }

    let Some(first_workload) = request.workloads.first() else {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Invalid,
            policy,
            vec!["workloads_empty".to_owned()],
        );
    };
    let unit_energy = first_workload.energy_kwh_per_work_unit;
    if !unit_energy.is_finite() || unit_energy <= 0.0 {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Invalid,
            policy,
            vec!["common_work_energy_coefficient_missing_or_invalid".to_owned()],
        );
    }
    let Some(scheduled_work_units) = capacity.max_deferrable_work_units else {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Invalid,
            policy,
            vec!["capacity_evaluator_returned_no_deferrable_total".to_owned()],
        );
    };

    let mut remaining = scheduled_work_units;
    let mut deferrable_work_units_by_interval = vec![0_u64; intervals.len()];
    for index in order {
        let here = per_interval_capacity[index].min(remaining);
        deferrable_work_units_by_interval[index] = here;
        remaining -= here;
    }
    if remaining != 0 {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Invalid,
            policy,
            vec!["schedule_capacity_did_not_cover_declared_total".to_owned()],
        );
    }

    let mut protected_floor_work_units_by_interval = vec![0_u64; intervals.len()];
    for workload in &request.workloads {
        if workload.work_class == WorkClass::Protected {
            for (index, units) in workload
                .minimum_accepted_units_by_interval
                .iter()
                .copied()
                .enumerate()
            {
                let Some(total) = protected_floor_work_units_by_interval[index].checked_add(units)
                else {
                    return DeferrableScheduleAssessment::no_schedule(
                        AssessmentStatus::Invalid,
                        policy,
                        vec!["protected_floor_interval_sum_overflow".to_owned()],
                    );
                };
                protected_floor_work_units_by_interval[index] = total;
            }
        }
    }

    let mut it_kwh = 0.0_f64;
    let mut facility_kwh = 0.0_f64;
    let mut cost_usd = 0.0_f64;
    let mut carbon_gco2 = 0.0_f64;
    let mut cost_known = true;
    let mut carbon_known = true;
    for (index, units) in deferrable_work_units_by_interval
        .iter()
        .copied()
        .enumerate()
    {
        let interval = &intervals[index];
        let Some(factor) = interval.facility_energy_per_it_energy else {
            return DeferrableScheduleAssessment::no_schedule(
                AssessmentStatus::Indeterminate,
                policy,
                vec![format!(
                    "interval_{}_facility_energy_factor_unknown",
                    interval.id
                )],
            );
        };
        let interval_it_kwh = units as f64 * unit_energy;
        let interval_facility_kwh = interval_it_kwh * factor;
        it_kwh += interval_it_kwh;
        facility_kwh += interval_facility_kwh;
        if units > 0 {
            if let Some(price) = interval.electricity_price_usd_per_kwh {
                cost_usd += interval_facility_kwh * price;
            } else {
                cost_known = false;
            }
            if let Some(intensity) = interval.grid_carbon_intensity_gco2_per_kwh {
                carbon_gco2 += interval_facility_kwh * intensity;
            } else {
                carbon_known = false;
            }
        }
    }
    if [it_kwh, facility_kwh, cost_usd, carbon_gco2]
        .iter()
        .any(|x| !x.is_finite())
    {
        return DeferrableScheduleAssessment::no_schedule(
            AssessmentStatus::Invalid,
            policy,
            vec!["schedule_derived_metric_non_finite".to_owned()],
        );
    }

    DeferrableScheduleAssessment {
        status: capacity.status,
        policy,
        protected_floor_work_units_by_interval: Some(protected_floor_work_units_by_interval),
        deferrable_work_units_by_interval: Some(deferrable_work_units_by_interval),
        scheduled_work_units: Some(scheduled_work_units),
        deferrable_it_energy_kwh: Some(it_kwh),
        deferrable_facility_energy_kwh: Some(facility_kwh),
        modelled_electricity_cost_usd: cost_known.then_some(cost_usd),
        modelled_grid_carbon_gco2: carbon_known.then_some(carbon_gco2),
        blockers: capacity.blockers,
    }
}

/// Evaluate every reference policy and return the non-dominated options on
/// the cost/carbon frontier. This is a frontier over the three declared
/// baselines only, not an optimizer over all feasible schedules.
pub fn compare_scheduling_policies(request: &Request) -> PolicyFrontierAssessment {
    let policies = [
        SchedulingPolicy::EarliestReady,
        SchedulingPolicy::LowestCost,
        SchedulingPolicy::LowestCarbon,
    ];
    let policy_assessments = policies
        .into_iter()
        .map(|policy| schedule_deferrable(request, policy))
        .collect::<Vec<_>>();

    let mut blockers = Vec::new();
    for candidate in &policy_assessments {
        for blocker in &candidate.blockers {
            blockers.push(format!("{:?}:{blocker}", candidate.policy));
        }
    }

    let status = if policy_assessments
        .iter()
        .any(|candidate| candidate.status == AssessmentStatus::Invalid)
    {
        Some(AssessmentStatus::Invalid)
    } else if policy_assessments
        .iter()
        .any(|candidate| candidate.status == AssessmentStatus::Unsupported)
    {
        Some(AssessmentStatus::Unsupported)
    } else if policy_assessments
        .iter()
        .any(|candidate| candidate.status == AssessmentStatus::Infeasible)
    {
        Some(AssessmentStatus::Infeasible)
    } else if policy_assessments
        .iter()
        .any(|candidate| candidate.status == AssessmentStatus::Indeterminate)
    {
        Some(AssessmentStatus::Indeterminate)
    } else {
        None
    };

    if let Some(status) = status {
        blockers.sort();
        blockers.dedup();
        return PolicyFrontierAssessment {
            status,
            policy_assessments,
            non_dominated_policies: Vec::new(),
            blockers,
        };
    }

    if policy_assessments.iter().any(|candidate| {
        candidate.modelled_electricity_cost_usd.is_none()
            || candidate.modelled_grid_carbon_gco2.is_none()
    }) {
        blockers.push("pareto_frontier_requires_complete_cost_and_carbon_attribution".to_owned());
        blockers.sort();
        blockers.dedup();
        return PolicyFrontierAssessment {
            status: AssessmentStatus::Indeterminate,
            policy_assessments,
            non_dominated_policies: Vec::new(),
            blockers,
        };
    }

    let mut non_dominated_policies = Vec::new();
    for (index, candidate) in policy_assessments.iter().enumerate() {
        let candidate_cost = candidate
            .modelled_electricity_cost_usd
            .expect("objective completeness checked above");
        let candidate_carbon = candidate
            .modelled_grid_carbon_gco2
            .expect("objective completeness checked above");
        let dominated = policy_assessments
            .iter()
            .enumerate()
            .any(|(other_index, other)| {
                if index == other_index {
                    return false;
                }
                let other_cost = other
                    .modelled_electricity_cost_usd
                    .expect("objective completeness checked above");
                let other_carbon = other
                    .modelled_grid_carbon_gco2
                    .expect("objective completeness checked above");
                other_cost <= candidate_cost
                    && other_carbon <= candidate_carbon
                    && (other_cost < candidate_cost || other_carbon < candidate_carbon)
            });
        if !dominated {
            non_dominated_policies.push(candidate.policy);
        }
    }

    blockers.sort();
    blockers.dedup();
    let status = if policy_assessments
        .iter()
        .any(|candidate| candidate.status == AssessmentStatus::CapacityLimited)
    {
        AssessmentStatus::CapacityLimited
    } else {
        AssessmentStatus::Feasible
    };
    PolicyFrontierAssessment {
        status,
        policy_assessments,
        non_dominated_policies,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_request() -> Request {
        // Parse the exact committed JSON fixture instead of maintaining a
        // second hand-copied request in Rust. Extra scenario-envelope fields
        // are intentionally ignored by Request; this evaluator consumes only
        // its bounded site/interval/workload projection.
        serde_json::from_str(include_str!(
            "../../schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json"
        ))
        .expect("committed synthetic compute-energy fixture must deserialize")
    }

    #[test]
    fn policy_comparison_preserves_cost_carbon_pareto_tradeoff() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 40;

        let comparison = compare_scheduling_policies(&request);
        assert_eq!(comparison.status, AssessmentStatus::Feasible);
        assert_eq!(comparison.policy_assessments.len(), 3);
        assert_eq!(
            comparison.non_dominated_policies,
            vec![SchedulingPolicy::LowestCost, SchedulingPolicy::LowestCarbon]
        );
        assert!(
            comparison
                .policy_assessments
                .iter()
                .all(|candidate| candidate.scheduled_work_units == Some(40))
        );
    }

    #[test]
    fn policy_comparison_withholds_frontier_when_tariff_data_is_missing() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 40;
        request.intervals[0].electricity_price_usd_per_kwh = None;

        let comparison = compare_scheduling_policies(&request);
        assert_eq!(comparison.status, AssessmentStatus::Indeterminate);
        assert!(comparison.non_dominated_policies.is_empty());
        assert_eq!(comparison.policy_assessments.len(), 3);
        assert!(
            comparison
                .blockers
                .iter()
                .any(|blocker| blocker.contains("interval_t0_electricity_price_unknown"))
        );
    }

    #[test]
    fn hand_calculated_fixture_has_66_deferrable_units() {
        let request = fixture_request();
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::CapacityLimited);
        assert_eq!(result.protected_floor_work_units, Some(24));
        assert_eq!(result.protected_floor_met, Some(true));
        assert_eq!(result.requested_deferrable_work_units, Some(80));
        assert_eq!(result.max_deferrable_work_units, Some(66));
        assert_eq!(
            result
                .interval_assessments
                .iter()
                .map(|item| item.max_deferrable_work_units)
                .collect::<Vec<_>>(),
            vec![28, 8, 2, 28]
        );
        assert!(result.interval_assessments.iter().all(|item| {
            item.it_energy_capacity_kwh + NUMERIC_TOLERANCE >= item.protected_floor_energy_kwh
        }));
        let expected = request
            .oracle_expectations
            .as_ref()
            .expect("committed fixture should declare its independent oracle values");
        assert_eq!(
            result.protected_floor_work_units,
            Some(expected.protected_work_units)
        );
        assert_eq!(
            result.max_deferrable_work_units,
            Some(expected.max_deferrable_work_units)
        );
        assert!(expected.resource_limits_hold);
    }

    #[test]
    fn unknown_water_intensity_is_indeterminate_not_zero() {
        let mut request = fixture_request();
        request.intervals[2].water_l_per_it_kwh = None;
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Indeterminate);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| item == "interval_t2_water_intensity_unknown")
        );
    }

    #[test]
    fn protected_floor_over_capacity_has_no_deferrable_plan() {
        let mut request = fixture_request();
        request.intervals[1].grid_import_limit_kw = Some(2.0);
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Infeasible);
        assert_eq!(result.protected_floor_met, Some(false));
        assert_eq!(result.max_deferrable_work_units, None);
    }

    #[test]
    fn misaligned_floor_vector_is_invalid() {
        let mut request = fixture_request();
        request.workloads[0]
            .minimum_accepted_units_by_interval
            .pop();
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Invalid);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| item == "workload_protected-inference_floor_length_mismatch")
        );
    }

    #[test]
    fn heterogeneous_work_energy_is_explicitly_unsupported() {
        let mut request = fixture_request();
        request.workloads[1].energy_kwh_per_work_unit = 0.25;
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Unsupported);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| item == "heterogeneous_work_energy_coefficients_unsupported_v0_1")
        );
    }

    #[test]
    fn duplicate_interval_identity_is_invalid() {
        let mut request = fixture_request();
        request.intervals[1].id = "t0".to_owned();
        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Invalid);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| item == "interval_id_duplicate:t0")
        );
    }

    #[test]
    fn lowest_cost_schedule_shifts_flexible_work_without_changing_protected_floors() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 40;

        let earliest = schedule_deferrable(&request, SchedulingPolicy::EarliestReady);
        let lowest_cost = schedule_deferrable(&request, SchedulingPolicy::LowestCost);

        assert_eq!(earliest.status, AssessmentStatus::Feasible);
        assert_eq!(lowest_cost.status, AssessmentStatus::Feasible);
        assert_eq!(
            earliest.deferrable_work_units_by_interval,
            Some(vec![28, 8, 2, 2])
        );
        assert_eq!(
            lowest_cost.deferrable_work_units_by_interval,
            Some(vec![12, 0, 0, 28])
        );
        assert_eq!(
            lowest_cost.protected_floor_work_units_by_interval,
            Some(vec![6, 6, 6, 6])
        );
        let baseline_cost = earliest.modelled_electricity_cost_usd.unwrap();
        let optimized_cost = lowest_cost.modelled_electricity_cost_usd.unwrap();
        assert!((baseline_cost - 3.7).abs() < 1.0e-8);
        assert!((optimized_cost - 1.978).abs() < 1.0e-8);
        assert!(optimized_cost < baseline_cost);
        assert_eq!(lowest_cost.scheduled_work_units, Some(40));
    }

    #[test]
    fn lowest_carbon_schedule_differs_from_lowest_cost_schedule() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 40;

        let lowest_cost = schedule_deferrable(&request, SchedulingPolicy::LowestCost);
        let lowest_carbon = schedule_deferrable(&request, SchedulingPolicy::LowestCarbon);

        assert_eq!(
            lowest_cost.deferrable_work_units_by_interval,
            Some(vec![12, 0, 0, 28])
        );
        assert_eq!(
            lowest_carbon.deferrable_work_units_by_interval,
            Some(vec![2, 8, 2, 28])
        );
        assert!(
            lowest_carbon.modelled_grid_carbon_gco2.unwrap()
                < lowest_cost.modelled_grid_carbon_gco2.unwrap()
        );
        assert_eq!(lowest_carbon.scheduled_work_units, Some(40));
    }

    #[test]
    fn carbon_policy_with_missing_factor_returns_indeterminate_without_schedule() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 40;
        request.intervals[2].grid_carbon_intensity_gco2_per_kwh = None;

        let result = schedule_deferrable(&request, SchedulingPolicy::LowestCarbon);
        assert_eq!(result.status, AssessmentStatus::Indeterminate);
        assert_eq!(result.deferrable_work_units_by_interval, None);
        assert_eq!(result.scheduled_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| { item == "interval_t2_grid_carbon_intensity_unknown" })
        );
    }

    #[test]
    fn lowest_cost_policy_accepts_negative_wholesale_price() {
        let mut request = fixture_request();
        request.workloads[1].required_work_units = 10;
        request.intervals[2].electricity_price_usd_per_kwh = Some(-0.05);

        let result = schedule_deferrable(&request, SchedulingPolicy::LowestCost);
        assert_eq!(result.status, AssessmentStatus::Feasible);
        assert_eq!(
            result.deferrable_work_units_by_interval,
            Some(vec![0, 0, 2, 8])
        );
        assert!((result.modelled_electricity_cost_usd.unwrap() - 0.298).abs() < 1.0e-8);
    }

    fn exhaustive_minimum_objective(
        capacities: &[u64],
        per_unit_objective: &[f64],
        demand: u64,
    ) -> f64 {
        fn visit(
            index: usize,
            remaining: u64,
            capacities: &[u64],
            per_unit_objective: &[f64],
            allocation: &mut Vec<u64>,
            best: &mut f64,
        ) {
            if index == capacities.len() {
                if remaining == 0 {
                    let objective = allocation
                        .iter()
                        .zip(per_unit_objective)
                        .map(|(units, unit_value)| *units as f64 * unit_value)
                        .sum::<f64>();
                    *best = (*best).min(objective);
                }
                return;
            }

            for units in 0..=capacities[index].min(remaining) {
                allocation.push(units);
                visit(
                    index + 1,
                    remaining - units,
                    capacities,
                    per_unit_objective,
                    allocation,
                    best,
                );
                allocation.pop();
            }
        }

        assert_eq!(capacities.len(), per_unit_objective.len());
        let mut best = f64::INFINITY;
        visit(
            0,
            demand,
            capacities,
            per_unit_objective,
            &mut Vec::with_capacity(capacities.len()),
            &mut best,
        );
        best
    }

    #[test]
    fn lowest_cost_policy_matches_exhaustive_oracle_across_tariff_profiles() {
        let tariff_profiles: [[f64; 4]; 6] = [
            [0.10, 0.30, 0.27, 0.08],
            [0.30, 0.10, 0.20, 0.40],
            [-0.05, 0.10, 0.15, 0.00],
            [0.20, -0.05, 0.20, -0.05],
            [0.20, 0.17037037037037036, 0.16428571428571428, 0.20],
            [0.75, 0.10, 0.09, 1.25],
        ];

        for prices in tariff_profiles {
            for demand in 1..=20 {
                let mut request = fixture_request();
                request.workloads[1].required_work_units = demand;
                for (interval, price) in request.intervals.iter_mut().zip(prices) {
                    interval.electricity_price_usd_per_kwh = Some(price);
                }

                let capacity = assess(&request);
                assert!(matches!(
                    capacity.status,
                    AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited
                ));
                let capacities = capacity
                    .interval_assessments
                    .iter()
                    .map(|row| row.max_deferrable_work_units)
                    .collect::<Vec<_>>();
                let unit_energy = request.workloads[0].energy_kwh_per_work_unit;
                let per_unit_cost = request
                    .intervals
                    .iter()
                    .zip(prices)
                    .map(|(interval, price)| {
                        price
                            * interval
                                .facility_energy_per_it_energy
                                .expect("fixture declares facility factors")
                            * unit_energy
                    })
                    .collect::<Vec<_>>();
                let oracle = exhaustive_minimum_objective(&capacities, &per_unit_cost, demand);
                let result = schedule_deferrable(&request, SchedulingPolicy::LowestCost);
                let allocation = result
                    .deferrable_work_units_by_interval
                    .as_ref()
                    .expect("finite tariff profile must produce a schedule");

                assert_eq!(result.status, AssessmentStatus::Feasible);
                assert_eq!(allocation.iter().sum::<u64>(), demand);
                assert!(
                    allocation
                        .iter()
                        .zip(&capacities)
                        .all(|(used, cap)| used <= cap)
                );
                assert_eq!(
                    result.protected_floor_work_units_by_interval,
                    Some(vec![6, 6, 6, 6])
                );
                assert!(
                    (result.modelled_electricity_cost_usd.expect("known tariffs") - oracle).abs()
                        < 1.0e-8,
                    "scheduler objective must equal exhaustive optimum; demand={demand}, prices={prices:?}, allocation={allocation:?}"
                );
                assert_eq!(
                    schedule_deferrable(&request, SchedulingPolicy::LowestCost)
                        .deferrable_work_units_by_interval
                        .as_ref(),
                    Some(allocation),
                    "identical request bytes must produce the same allocation"
                );
            }
        }
    }

    #[test]
    fn lowest_carbon_policy_matches_exhaustive_oracle_across_intensity_profiles() {
        let intensity_profiles: [[f64; 4]; 6] = [
            [1200.0, 700.0, 600.0, 250.0],
            [250.0, 600.0, 700.0, 1200.0],
            [0.0, 100.0, 300.0, 500.0],
            [100.0, 0.0, 100.0, 0.0],
            [100.0, 125.0, 150.0, 175.0],
            [800.0, 200.0, 200.0, 800.0],
        ];

        for intensities in intensity_profiles {
            for demand in 1..=20 {
                let mut request = fixture_request();
                request.workloads[1].required_work_units = demand;
                for (interval, intensity) in request.intervals.iter_mut().zip(intensities) {
                    interval.grid_carbon_intensity_gco2_per_kwh = Some(intensity);
                }

                let capacity = assess(&request);
                assert!(matches!(
                    capacity.status,
                    AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited
                ));
                let capacities = capacity
                    .interval_assessments
                    .iter()
                    .map(|row| row.max_deferrable_work_units)
                    .collect::<Vec<_>>();
                let unit_energy = request.workloads[0].energy_kwh_per_work_unit;
                let per_unit_emissions = request
                    .intervals
                    .iter()
                    .zip(intensities)
                    .map(|(interval, intensity)| {
                        intensity
                            * interval
                                .facility_energy_per_it_energy
                                .expect("fixture declares facility factors")
                            * unit_energy
                    })
                    .collect::<Vec<_>>();
                let oracle = exhaustive_minimum_objective(&capacities, &per_unit_emissions, demand);
                let result = schedule_deferrable(&request, SchedulingPolicy::LowestCarbon);
                let allocation = result
                    .deferrable_work_units_by_interval
                    .as_ref()
                    .expect("finite carbon profile must produce a schedule");

                assert_eq!(result.status, AssessmentStatus::Feasible);
                assert_eq!(allocation.iter().sum::<u64>(), demand);
                assert!(
                    allocation
                        .iter()
                        .zip(&capacities)
                        .all(|(used, cap)| used <= cap)
                );
                assert_eq!(
                    result.protected_floor_work_units_by_interval,
                    Some(vec![6, 6, 6, 6])
                );
                assert!(
                    (result
                        .modelled_grid_carbon_gco2
                        .expect("known intensity factors")
                        - oracle)
                        .abs()
                        < 1.0e-7,
                    "carbon scheduler must match exhaustive optimum; demand={demand}, intensities={intensities:?}, allocation={allocation:?}"
                );
                assert_eq!(
                    schedule_deferrable(&request, SchedulingPolicy::LowestCarbon)
                        .deferrable_work_units_by_interval
                        .as_ref(),
                    Some(allocation),
                    "identical request must yield identical carbon allocation"
                );
            }
        }
    }

    #[test]
    fn deterministic_resource_scenarios_match_cost_and_carbon_exhaustive_oracles() {
        fn next_bounded(seed: &mut u64, upper_exclusive: u64) -> u64 {
            *seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (*seed >> 32) % upper_exclusive
        }

        let mut seed = 0x5eed_cafe_d00d_f00d_u64;
        let mut checked_cases = 0_u64;
        for case_index in 0..64 {
            let mut request = fixture_request();
            let mut generated_capacities = [
                next_bounded(&mut seed, 5),
                next_bounded(&mut seed, 5),
                next_bounded(&mut seed, 5),
                next_bounded(&mut seed, 5),
            ];
            if generated_capacities.iter().sum::<u64>() == 0 {
                generated_capacities[case_index % 4] = 1;
            }
            let total_capacity = generated_capacities.iter().sum::<u64>();
            let demand = 1 + next_bounded(&mut seed, total_capacity);
            request.workloads[1].required_work_units = demand;

            for (interval, target_capacity) in
                request.intervals.iter_mut().zip(generated_capacities)
            {
                // The protected floor is six units × 0.5 kWh = 3 kWh in
                // every one-hour interval. Set water as the binding resource
                // so the residual capacity is exactly the generated integer.
                let protected_energy_kwh = 3.0;
                let deferrable_energy_kwh = target_capacity as f64 * 0.5;
                let water_intensity = interval
                    .water_l_per_it_kwh
                    .expect("fixture declares water intensity");
                interval.cooling_water_limit_l = Some(
                    (protected_energy_kwh + deferrable_energy_kwh)
                        * water_intensity
                        * interval.duration_hours,
                );
                interval.thermal_it_power_limit_kw = Some(20.0);
                interval.electricity_price_usd_per_kwh =
                    Some((next_bounded(&mut seed, 17) as i64 - 4) as f64 * 0.025);
                interval.grid_carbon_intensity_gco2_per_kwh =
                    Some(next_bounded(&mut seed, 1201) as f64);
            }

            let capacity = assess(&request);
            assert!(
                matches!(
                    capacity.status,
                    AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited
                ),
                "capacity precondition failed in generated case {case_index}: {:?}",
                capacity.blockers
            );
            let derived_capacities = capacity
                .interval_assessments
                .iter()
                .map(|row| row.max_deferrable_work_units)
                .collect::<Vec<_>>();
            assert_eq!(
                derived_capacities, generated_capacities,
                "water-derived capacities differ from the generated oracle in case {case_index}"
            );
            let unit_energy = request.workloads[0].energy_kwh_per_work_unit;

            let per_unit_cost = request
                .intervals
                .iter()
                .map(|interval| {
                    interval
                        .electricity_price_usd_per_kwh
                        .expect("price assigned")
                        * interval
                            .facility_energy_per_it_energy
                            .expect("fixture declares facility factor")
                        * unit_energy
                })
                .collect::<Vec<_>>();
            let cost_oracle =
                exhaustive_minimum_objective(&generated_capacities, &per_unit_cost, demand);
            let cost_schedule = schedule_deferrable(&request, SchedulingPolicy::LowestCost);
            let cost_allocation = cost_schedule
                .deferrable_work_units_by_interval
                .as_ref()
                .expect("generated cost case must schedule");
            assert_eq!(cost_allocation.iter().sum::<u64>(), demand);
            assert!(
                (cost_schedule
                    .modelled_electricity_cost_usd
                    .expect("price known")
                    - cost_oracle)
                    .abs()
                    < 1.0e-7,
                "cost oracle disagreement in case {case_index}"
            );

            let per_unit_carbon = request
                .intervals
                .iter()
                .map(|interval| {
                    interval
                        .grid_carbon_intensity_gco2_per_kwh
                        .expect("carbon intensity assigned")
                        * interval
                            .facility_energy_per_it_energy
                            .expect("fixture declares facility factor")
                        * unit_energy
                })
                .collect::<Vec<_>>();
            let carbon_oracle =
                exhaustive_minimum_objective(&generated_capacities, &per_unit_carbon, demand);
            let carbon_schedule = schedule_deferrable(&request, SchedulingPolicy::LowestCarbon);
            let carbon_allocation = carbon_schedule
                .deferrable_work_units_by_interval
                .as_ref()
                .expect("generated carbon case must schedule");
            assert_eq!(carbon_allocation.iter().sum::<u64>(), demand);
            assert!(
                (carbon_schedule
                    .modelled_grid_carbon_gco2
                    .expect("carbon intensity known")
                    - carbon_oracle)
                    .abs()
                    < 1.0e-7,
                "carbon oracle disagreement in case {case_index}"
            );

            assert_eq!(
                cost_schedule.protected_floor_work_units_by_interval,
                Some(vec![6, 6, 6, 6])
            );
            assert_eq!(
                carbon_schedule.protected_floor_work_units_by_interval,
                Some(vec![6, 6, 6, 6])
            );
            checked_cases += 2;
        }
        assert_eq!(checked_cases, 128);
    }

    #[test]
    fn non_finite_derived_objective_rank_fails_closed_before_allocation() {
        for policy in [SchedulingPolicy::LowestCost, SchedulingPolicy::LowestCarbon] {
            let mut request = fixture_request();
            request.workloads[1].required_work_units = 1;
            // Raw input is finite, but value × facility factor overflows in t2.
            // That interval need not receive work for its bad rank to corrupt ordering.
            match policy {
                SchedulingPolicy::LowestCost => {
                    request.intervals[2].electricity_price_usd_per_kwh = Some(1.7e308);
                }
                SchedulingPolicy::LowestCarbon => {
                    request.intervals[2].grid_carbon_intensity_gco2_per_kwh = Some(1.7e308);
                }
                SchedulingPolicy::EarliestReady => unreachable!("only ranked policies are tested"),
            }

            let result = schedule_deferrable(&request, policy);
            assert_eq!(result.status, AssessmentStatus::Invalid);
            assert_eq!(result.deferrable_work_units_by_interval, None);
            assert!(
                result
                    .blockers
                    .iter()
                    .any(|blocker| blocker == "interval_t2_derived_objective_per_unit_non_finite")
            );
        }
    }

    #[test]
    fn storage_profile_is_rejected_until_dispatch_is_modelled() {
        let mut request = fixture_request();
        request.storage = Some(StorageProfile {
            capacity_kwh: 20.0,
            opening_stock_kwh: 10.0,
            charge_limit_kw: 5.0,
            discharge_limit_kw: 5.0,
            round_trip_efficiency: 0.9,
            terminal_reserve_kwh: 2.0,
        });

        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Unsupported);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result
                .blockers
                .iter()
                .any(|item| item == "storage_dispatch_unsupported_v0_1")
        );
    }

    #[test]
    fn non_interruptible_deferrable_work_is_not_silently_split() {
        let mut request = fixture_request();
        request.workloads[1].interruptible = false;

        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Unsupported);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result.blockers.iter().any(|item| {
                item == "workload_deferrable-batch_deferrable_not_interruptible_v0_1"
            })
        );
    }

    #[test]
    fn heterogeneous_deferrable_quality_profiles_are_unsupported() {
        let mut request = fixture_request();
        let mut other = request.workloads[1].clone();
        other.workload_id = "different-quality-batch".to_owned();
        other.required_work_units = 1;
        other.service_quality_profile_id = "different-quality-profile".to_owned();
        request.workloads.push(other);

        let result = assess(&request);
        assert_eq!(result.status, AssessmentStatus::Unsupported);
        assert_eq!(result.max_deferrable_work_units, None);
        assert!(
            result.blockers.iter().any(|item| {
                item == "heterogeneous_deferrable_quality_profiles_unsupported_v0_1"
            })
        );
    }

    #[test]
    fn unknown_nested_scenario_fields_fail_deserialization() {
        let mut value: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json"
        ))
        .expect("fixture should be valid JSON");
        value["intervals"][0]["unmodelled_grid_outage"] = serde_json::json!(true);

        assert!(serde_json::from_value::<Request>(value).is_err());
    }
}
