//! Deterministic essential-service continuity and restoration evaluator.
//!
//! This crate evaluates already-qualified service demand/delivery trajectories.
//! It does not simulate the physical systems that produce those trajectories.
//!
//! Time is event-relative elapsed seconds from t0, not wall-clock UTC. This
//! keeps outage/recovery arithmetic usable when GNSS or external time is absent.
//! Source-time trust and absolute-time provenance remain separate evidence.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Provenance class for a complete service trajectory artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceClass {
    Measured,
    PublishedDerived,
    Modelled,
    Inferred,
    Simulated,
}

/// One interval of required and delivered service quantity.
///
/// Quantities are totals over [start_offset_seconds, end_offset_seconds), in
/// the native unit declared by the containing ServiceSeries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServiceInterval {
    pub start_offset_seconds: u64,
    pub end_offset_seconds: u64,
    pub required_quantity: f64,
    pub delivered_quantity: f64,
}

/// One service's complete event-horizon trajectory and evidence binding.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceSeries {
    pub service_id: String,
    pub quantity_unit: String,
    pub asset_revision: String,
    pub source_ref: String,
    /// SHA-256 digest of the canonical input trajectory artifact, lowercase hex.
    pub source_sha256: String,
    pub evidence_class: EvidenceClass,
    /// Owner-declared minimum fraction of required service (0, 1].
    pub minimum_service_fraction: f64,
    /// Absolute tolerance in the service's native quantity unit.
    pub quantity_tolerance: f64,
    /// A recovery must remain above threshold for this many consecutive seconds.
    pub recovery_hold_seconds: u64,
    pub intervals: Vec<ServiceInterval>,
}

/// One event/scenario, with all trajectories covering exactly [0, horizon).
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceScenario {
    pub scenario_id: String,
    pub horizon_seconds: u64,
    pub services: Vec<ServiceSeries>,
}

/// Beginning and confirmation time for a sustained recovery.
///
/// The beginning field is when the qualifying run started. The confirmation
/// field is when enough consecutive evidence accumulated to confirm the hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryMark {
    pub began_after_seconds: u64,
    pub confirmed_after_seconds: u64,
}

/// Per-service resilience vector. Every quantity remains in the service's
/// declared native unit; no cross-service scalar score is produced.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceMetrics {
    pub service_id: String,
    pub quantity_unit: String,
    pub asset_revision: String,
    pub source_ref: String,
    pub source_sha256: String,
    pub evidence_class: EvidenceClass,
    pub horizon_seconds: u64,
    pub initially_above_service_floor: bool,
    pub initially_fully_served: bool,
    pub total_required_quantity: f64,
    pub total_delivered_quantity_capped: f64,
    pub total_surplus_quantity_excluded: f64,
    pub total_unserved_quantity: f64,
    pub demand_observed_seconds: u64,
    pub demand_weighted_served_fraction: f64,
    pub floor_time_availability_fraction: f64,
    pub total_floor_violation_seconds: u64,
    pub longest_floor_violation_seconds: u64,
    pub floor_violation_episode_count: u64,
    pub maximum_interval_deficit_fraction: f64,
    pub first_floor_violation_after_seconds: Option<u64>,
    pub floor_recovery: Option<RecoveryMark>,
    pub full_service_recovery: Option<RecoveryMark>,
}

/// Complete evaluation. The result is deliberately a vector of services.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioEvaluation {
    pub scenario_id: String,
    pub horizon_seconds: u64,
    pub services: Vec<ServiceMetrics>,
}

/// Fail-closed input and numerical errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceError {
    EmptyScenarioId,
    InvalidHorizon,
    EmptyServiceSet,
    EmptyIdentifier,
    DuplicateServiceId,
    EmptyUnit,
    EmptySourceReference,
    EmptyAssetRevision,
    InvalidSourceDigest,
    InvalidServiceFloor,
    InvalidQuantityTolerance,
    InvalidRecoveryHold,
    EmptyIntervals,
    InvalidInterval,
    IncompleteOrNonContiguousCoverage,
    NonFiniteQuantity,
    NegativeQuantity,
    NoPositiveDemand,
    NumericalOverflow,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyScenarioId => "scenario ID must not be empty",
            Self::InvalidHorizon => "scenario horizon must be positive",
            Self::EmptyServiceSet => "scenario must contain at least one service",
            Self::EmptyIdentifier => "service ID must be non-empty and trimmed",
            Self::DuplicateServiceId => "service IDs must be unique within a scenario",
            Self::EmptyUnit => "service quantity unit must be non-empty and trimmed",
            Self::EmptySourceReference => "source reference must be non-empty and trimmed",
            Self::EmptyAssetRevision => "asset revision must be non-empty and trimmed",
            Self::InvalidSourceDigest => "SHA-256 digest must be 64 lowercase hex characters",
            Self::InvalidServiceFloor => "minimum service fraction must be finite and in (0, 1]",
            Self::InvalidQuantityTolerance => "tolerance must be finite, non-negative, and below each positive-demand floor",
            Self::InvalidRecoveryHold => "recovery hold duration must be positive",
            Self::EmptyIntervals => "service trajectory must contain at least one interval",
            Self::InvalidInterval => "interval duration must be positive and within horizon",
            Self::IncompleteOrNonContiguousCoverage => "intervals must cover [0, horizon) contiguously",
            Self::NonFiniteQuantity => "required and delivered quantities must be finite",
            Self::NegativeQuantity => "required and delivered quantities must be non-negative",
            Self::NoPositiveDemand => "a service trajectory must contain positive total demand",
            Self::NumericalOverflow => "metric accumulation overflowed or became non-finite",
        };
        formatter.write_str(message)
    }
}

impl Error for ServiceError {}

/// Evaluate all services independently, rejecting incomplete event coverage.
pub fn evaluate_scenario(
    scenario: &ServiceScenario,
) -> Result<ScenarioEvaluation, ServiceError> {
    if scenario.scenario_id.trim().is_empty()
        || scenario.scenario_id.trim() != scenario.scenario_id
    {
        return Err(ServiceError::EmptyScenarioId);
    }
    if scenario.horizon_seconds == 0 {
        return Err(ServiceError::InvalidHorizon);
    }
    if scenario.services.is_empty() {
        return Err(ServiceError::EmptyServiceSet);
    }

    let mut seen = BTreeSet::new();
    let mut results = Vec::with_capacity(scenario.services.len());

    for service in &scenario.services {
        if !seen.insert(service.service_id.as_str()) {
            return Err(ServiceError::DuplicateServiceId);
        }
        results.push(evaluate_service(service, scenario.horizon_seconds)?);
    }

    Ok(ScenarioEvaluation {
        scenario_id: scenario.scenario_id.clone(),
        horizon_seconds: scenario.horizon_seconds,
        services: results,
    })
}

/// Evaluate one service trajectory without combining its units with other
/// services. Intervals must already be ordered and contiguous.
pub fn evaluate_service(
    service: &ServiceSeries,
    horizon_seconds: u64,
) -> Result<ServiceMetrics, ServiceError> {
    validate_service_metadata(service)?;
    if horizon_seconds == 0 {
        return Err(ServiceError::InvalidHorizon);
    }
    validate_intervals(service, horizon_seconds)?;

    let mut total_required = 0.0;
    let mut total_delivered_capped = 0.0;
    let mut total_surplus = 0.0;
    let mut total_unserved = 0.0;
    let mut floor_available_seconds = 0_u64;
    let mut demand_seconds = 0_u64;
    let mut total_floor_violation_seconds = 0_u64;
    let mut current_floor_violation_seconds = 0_u64;
    let mut longest_floor_violation_seconds = 0_u64;
    let mut floor_violation_episode_count = 0_u64;
    let mut maximum_interval_deficit_fraction = 0.0_f64;
    let mut first_floor_violation_after_seconds = None;

    let mut floor_status = Vec::with_capacity(service.intervals.len());
    let mut full_status = Vec::with_capacity(service.intervals.len());

    for interval in &service.intervals {
        let duration = interval.end_offset_seconds - interval.start_offset_seconds;
        let required = interval.required_quantity;
        let delivered = interval.delivered_quantity;
        let capped_delivered = delivered.min(required);
        let unserved = (required - delivered).max(0.0);
        let surplus = (delivered - required).max(0.0);

        total_required += required;
        total_delivered_capped += capped_delivered;
        total_unserved += unserved;
        total_surplus += surplus;

        if !all_finite(&[
            total_required,
            total_delivered_capped,
            total_unserved,
            total_surplus,
        ]) {
            return Err(ServiceError::NumericalOverflow);
        }

        if required == 0.0 {
            floor_status.push(None);
            full_status.push(None);
            longest_floor_violation_seconds =
                longest_floor_violation_seconds.max(current_floor_violation_seconds);
            current_floor_violation_seconds = 0;
            continue;
        }

        demand_seconds = demand_seconds
            .checked_add(duration)
            .ok_or(ServiceError::NumericalOverflow)?;
        let served_fraction = (capped_delivered / required).clamp(0.0, 1.0);
        let deficit_fraction = (1.0 - served_fraction).clamp(0.0, 1.0);
        let floor_threshold =
            required * service.minimum_service_fraction - service.quantity_tolerance;
        let full_threshold = required - service.quantity_tolerance;
        let floor_met = delivered >= floor_threshold;
        let full_met = delivered >= full_threshold;

        if !served_fraction.is_finite() || !deficit_fraction.is_finite() {
            return Err(ServiceError::NumericalOverflow);
        }
        maximum_interval_deficit_fraction =
            maximum_interval_deficit_fraction.max(deficit_fraction);

        floor_status.push(Some(floor_met));
        full_status.push(Some(full_met));

        if floor_met {
            floor_available_seconds = floor_available_seconds
                .checked_add(duration)
                .ok_or(ServiceError::NumericalOverflow)?;
            longest_floor_violation_seconds =
                longest_floor_violation_seconds.max(current_floor_violation_seconds);
            current_floor_violation_seconds = 0;
        } else {
            if current_floor_violation_seconds == 0 {
                floor_violation_episode_count = floor_violation_episode_count
                    .checked_add(1)
                    .ok_or(ServiceError::NumericalOverflow)?;
                first_floor_violation_after_seconds
                    .get_or_insert(interval.start_offset_seconds);
            }
            current_floor_violation_seconds = current_floor_violation_seconds
                .checked_add(duration)
                .ok_or(ServiceError::NumericalOverflow)?;
            total_floor_violation_seconds = total_floor_violation_seconds
                .checked_add(duration)
                .ok_or(ServiceError::NumericalOverflow)?;
            longest_floor_violation_seconds = longest_floor_violation_seconds
                .max(current_floor_violation_seconds);
        }
    }

    if total_required <= 0.0 {
        return Err(ServiceError::NoPositiveDemand);
    }

    let demand_weighted_served_fraction = total_delivered_capped / total_required;
    let floor_time_availability_fraction =
        floor_available_seconds as f64 / demand_seconds as f64;
    if !all_finite(&[
        demand_weighted_served_fraction,
        floor_time_availability_fraction,
        maximum_interval_deficit_fraction,
    ]) {
        return Err(ServiceError::NumericalOverflow);
    }

    Ok(ServiceMetrics {
        service_id: service.service_id.clone(),
        quantity_unit: service.quantity_unit.clone(),
        asset_revision: service.asset_revision.clone(),
        source_ref: service.source_ref.clone(),
        source_sha256: service.source_sha256.clone(),
        evidence_class: service.evidence_class,
        horizon_seconds,
        initially_above_service_floor: floor_status.iter().find_map(|status| *status).ok_or(
            ServiceError::NoPositiveDemand,
        )?,
        initially_fully_served: full_status.iter().find_map(|status| *status).ok_or(
            ServiceError::NoPositiveDemand,
        )?,
        total_required_quantity: total_required,
        total_delivered_quantity_capped: total_delivered_capped,
        total_surplus_quantity_excluded: total_surplus,
        total_unserved_quantity: total_unserved,
        demand_observed_seconds: demand_seconds,
        demand_weighted_served_fraction,
        floor_time_availability_fraction,
        total_floor_violation_seconds,
        longest_floor_violation_seconds,
        floor_violation_episode_count,
        maximum_interval_deficit_fraction,
        first_floor_violation_after_seconds,
        floor_recovery: find_sustained_recovery(
            &service.intervals,
            &floor_status,
            service.recovery_hold_seconds,
        ),
        full_service_recovery: find_sustained_recovery(
            &service.intervals,
            &full_status,
            service.recovery_hold_seconds,
        ),
    })
}

fn validate_service_metadata(service: &ServiceSeries) -> Result<(), ServiceError> {
    if service.service_id.trim().is_empty()
        || service.service_id.trim() != service.service_id
    {
        return Err(ServiceError::EmptyIdentifier);
    }
    if service.quantity_unit.trim().is_empty()
        || service.quantity_unit.trim() != service.quantity_unit
    {
        return Err(ServiceError::EmptyUnit);
    }
    if service.asset_revision.trim().is_empty()
        || service.asset_revision.trim() != service.asset_revision
    {
        return Err(ServiceError::EmptyAssetRevision);
    }
    if service.source_ref.trim().is_empty()
        || service.source_ref.trim() != service.source_ref
    {
        return Err(ServiceError::EmptySourceReference);
    }
    if service.source_sha256.len() != 64
        || !service
            .source_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ServiceError::InvalidSourceDigest);
    }
    if !service.minimum_service_fraction.is_finite()
        || service.minimum_service_fraction <= 0.0
        || service.minimum_service_fraction > 1.0
    {
        return Err(ServiceError::InvalidServiceFloor);
    }
    if !service.quantity_tolerance.is_finite() || service.quantity_tolerance < 0.0 {
        return Err(ServiceError::InvalidQuantityTolerance);
    }
    if service.recovery_hold_seconds == 0 {
        return Err(ServiceError::InvalidRecoveryHold);
    }
    if service.intervals.is_empty() {
        return Err(ServiceError::EmptyIntervals);
    }
    Ok(())
}

fn validate_intervals(
    service: &ServiceSeries,
    horizon_seconds: u64,
) -> Result<(), ServiceError> {
    let mut expected_start = 0_u64;

    for interval in &service.intervals {
        if interval.end_offset_seconds <= interval.start_offset_seconds
            || interval.end_offset_seconds > horizon_seconds
        {
            return Err(ServiceError::InvalidInterval);
        }
        if interval.start_offset_seconds != expected_start {
            return Err(ServiceError::IncompleteOrNonContiguousCoverage);
        }
        if !interval.required_quantity.is_finite()
            || !interval.delivered_quantity.is_finite()
        {
            return Err(ServiceError::NonFiniteQuantity);
        }
        if interval.required_quantity < 0.0 || interval.delivered_quantity < 0.0 {
            return Err(ServiceError::NegativeQuantity);
        }
        if interval.required_quantity > 0.0
            && service.quantity_tolerance
                >= interval.required_quantity * service.minimum_service_fraction
        {
            return Err(ServiceError::InvalidQuantityTolerance);
        }
        expected_start = interval.end_offset_seconds;
    }

    if expected_start != horizon_seconds {
        return Err(ServiceError::IncompleteOrNonContiguousCoverage);
    }
    Ok(())
}

fn find_sustained_recovery(
    intervals: &[ServiceInterval],
    status: &[Option<bool>],
    hold_seconds: u64,
) -> Option<RecoveryMark> {
    let mut observed_violation = false;
    let mut run_start = None;
    let mut run_duration = 0_u64;

    for (interval, state) in intervals.iter().zip(status.iter().copied()) {
        match state {
            None => {
                run_start = None;
                run_duration = 0;
            }
            Some(false) => {
                observed_violation = true;
                run_start = None;
                run_duration = 0;
            }
            Some(true) if observed_violation => {
                if run_start.is_none() {
                    run_start = Some(interval.start_offset_seconds);
                    run_duration = 0;
                }
                run_duration = run_duration.checked_add(
                    interval.end_offset_seconds - interval.start_offset_seconds,
                )?;

                if run_duration >= hold_seconds {
                    return Some(RecoveryMark {
                        began_after_seconds: run_start?,
                        confirmed_after_seconds: interval.end_offset_seconds,
                    });
                }
            }
            Some(true) => {}
        }
    }
    None
}

fn all_finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn service(intervals: Vec<ServiceInterval>) -> ServiceSeries {
        ServiceSeries {
            service_id: "hospital-critical-power".to_owned(),
            quantity_unit: "kWh".to_owned(),
            asset_revision: "hospital-power-r7".to_owned(),
            source_ref: "urn:test:scenario:power-trajectory".to_owned(),
            source_sha256: DIGEST.to_owned(),
            evidence_class: EvidenceClass::Simulated,
            minimum_service_fraction: 0.8,
            quantity_tolerance: 0.0,
            recovery_hold_seconds: 30,
            intervals,
        }
    }

    fn interval(start: u64, end: u64, required: f64, delivered: f64) -> ServiceInterval {
        ServiceInterval {
            start_offset_seconds: start,
            end_offset_seconds: end,
            required_quantity: required,
            delivered_quantity: delivered,
        }
    }

    fn scenario(services: Vec<ServiceSeries>, horizon_seconds: u64) -> ServiceScenario {
        ServiceScenario {
            scenario_id: "gmd-synthetic-case-01".to_owned(),
            horizon_seconds,
            services,
        }
    }

    #[test]
    fn full_service_reports_complete_floor_and_full_recovery() {
        let input = service(vec![interval(0, 30, 100.0, 100.0), interval(30, 60, 100.0, 100.0)]);
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(result.total_required_quantity, 200.0);
        assert_eq!(result.total_delivered_quantity_capped, 200.0);
        assert_eq!(result.total_unserved_quantity, 0.0);
        assert_eq!(result.demand_weighted_served_fraction, 1.0);
        assert_eq!(result.floor_time_availability_fraction, 1.0);
        assert_eq!(result.floor_violation_episode_count, 0);
        assert!(result.initially_above_service_floor);
        assert!(result.initially_fully_served);
        assert_eq!(result.floor_recovery, None);
        assert_eq!(result.full_service_recovery, None);
    }

    #[test]
    fn partial_delivery_reports_native_unit_deficit_and_recovery() {
        let input = service(vec![
            interval(0, 30, 100.0, 80.0),
            interval(30, 60, 100.0, 0.0),
            interval(60, 90, 100.0, 100.0),
        ]);
        let result = evaluate_service(&input, 90).unwrap();

        assert_eq!(result.total_unserved_quantity, 120.0);
        assert_eq!(result.demand_weighted_served_fraction, 0.6);
        assert_eq!(result.floor_time_availability_fraction, 2.0 / 3.0);
        assert_eq!(result.total_floor_violation_seconds, 30);
        assert_eq!(result.longest_floor_violation_seconds, 30);
        assert_eq!(result.maximum_interval_deficit_fraction, 1.0);
        assert_eq!(result.floor_violation_episode_count, 1);
        assert_eq!(result.first_floor_violation_after_seconds, Some(30));
        assert!(result.initially_above_service_floor);
        assert!(!result.initially_fully_served);
        assert_eq!(
            result.floor_recovery,
            Some(RecoveryMark {
                began_after_seconds: 60,
                confirmed_after_seconds: 90,
            })
        );
        assert_eq!(result.full_service_recovery, result.floor_recovery);
    }

    #[test]
    fn surplus_is_reported_but_never_credited_as_service() {
        let input = service(vec![interval(0, 60, 100.0, 150.0)]);
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(result.total_delivered_quantity_capped, 100.0);
        assert_eq!(result.total_surplus_quantity_excluded, 50.0);
        assert_eq!(result.demand_weighted_served_fraction, 1.0);
        assert_eq!(result.total_unserved_quantity, 0.0);
    }

    #[test]
    fn floor_recovery_is_distinct_from_full_service_recovery() {
        let input = service(vec![
            interval(0, 30, 100.0, 60.0),
            interval(30, 60, 100.0, 80.0),
            interval(60, 90, 100.0, 80.0),
        ]);
        let result = evaluate_service(&input, 90).unwrap();

        assert!(!result.initially_above_service_floor);
        assert_eq!(
            result.floor_recovery,
            Some(RecoveryMark {
                began_after_seconds: 30,
                confirmed_after_seconds: 60,
            })
        );
        assert_eq!(result.full_service_recovery, None);
    }

    #[test]
    fn transient_supply_spike_does_not_confirm_sustained_recovery() {
        let input = service(vec![
            interval(0, 20, 100.0, 20.0),
            interval(20, 30, 100.0, 100.0),
            interval(30, 50, 100.0, 20.0),
        ]);
        let result = evaluate_service(&input, 50).unwrap();

        assert_eq!(result.floor_recovery, None);
        assert_eq!(result.full_service_recovery, None);
    }

    #[test]
    fn recovery_is_confirmed_only_after_the_entire_hold_window() {
        let input = service(vec![
            interval(0, 15, 100.0, 0.0),
            interval(15, 30, 100.0, 100.0),
            interval(30, 45, 100.0, 100.0),
            interval(45, 60, 100.0, 100.0),
        ]);
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(
            result.floor_recovery,
            Some(RecoveryMark {
                began_after_seconds: 15,
                confirmed_after_seconds: 45,
            })
        );
    }

    #[test]
    fn irregular_contiguous_intervals_are_supported() {
        let input = service(vec![
            interval(0, 7, 100.0, 0.0),
            interval(7, 28, 100.0, 100.0),
            interval(28, 60, 100.0, 100.0),
        ]);
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(result.total_floor_violation_seconds, 7);
        assert_eq!(result.floor_time_availability_fraction, 53.0 / 60.0);
        assert_eq!(
            result.floor_recovery,
            Some(RecoveryMark {
                began_after_seconds: 7,
                confirmed_after_seconds: 60,
            })
        );
    }

    #[test]
    fn zero_demand_intervals_do_not_create_false_deficits() {
        let input = service(vec![
            interval(0, 30, 0.0, 0.0),
            interval(30, 60, 100.0, 100.0),
        ]);
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(result.floor_time_availability_fraction, 1.0);
        assert_eq!(result.demand_weighted_served_fraction, 1.0);
        assert_eq!(result.total_unserved_quantity, 0.0);
        assert!(result.initially_above_service_floor);
        assert_eq!(result.floor_recovery, None);
    }

    #[test]
    fn no_demand_interval_does_not_bridge_a_recovery_window() {
        let input = service(vec![
            interval(0, 10, 100.0, 0.0),
            interval(10, 40, 0.0, 0.0),
            interval(40, 60, 100.0, 100.0),
            interval(60, 90, 100.0, 100.0),
        ]);
        let result = evaluate_service(&input, 90).unwrap();

        assert_eq!(result.demand_observed_seconds, 60);
        assert_eq!(result.floor_time_availability_fraction, 50.0 / 60.0);
        assert_eq!(result.total_floor_violation_seconds, 10);
        assert_eq!(
            result.floor_recovery,
            Some(RecoveryMark {
                began_after_seconds: 40,
                confirmed_after_seconds: 90,
            })
        );
    }

    #[test]
    fn missing_or_overlapping_intervals_are_rejected() {
        let missing = service(vec![
            interval(0, 20, 100.0, 100.0),
            interval(30, 60, 100.0, 100.0),
        ]);
        assert_eq!(
            evaluate_service(&missing, 60),
            Err(ServiceError::IncompleteOrNonContiguousCoverage)
        );

        let overlapping = service(vec![
            interval(0, 40, 100.0, 100.0),
            interval(30, 60, 100.0, 100.0),
        ]);
        assert_eq!(
            evaluate_service(&overlapping, 60),
            Err(ServiceError::IncompleteOrNonContiguousCoverage)
        );
    }

    #[test]
    fn reordered_and_duplicated_intervals_are_rejected() {
        let reordered = service(vec![
            interval(30, 60, 100.0, 100.0),
            interval(0, 30, 100.0, 100.0),
        ]);
        assert_eq!(
            evaluate_service(&reordered, 60),
            Err(ServiceError::IncompleteOrNonContiguousCoverage)
        );

        let duplicated = service(vec![
            interval(0, 30, 100.0, 100.0),
            interval(0, 30, 100.0, 100.0),
        ]);
        assert_eq!(
            evaluate_service(&duplicated, 60),
            Err(ServiceError::IncompleteOrNonContiguousCoverage)
        );
    }

    #[test]
    fn non_finite_and_negative_quantities_fail_closed() {
        let non_finite = service(vec![interval(0, 60, 100.0, f64::NAN)]);
        assert_eq!(
            evaluate_service(&non_finite, 60),
            Err(ServiceError::NonFiniteQuantity)
        );

        let negative = service(vec![interval(0, 60, 100.0, -1.0)]);
        assert_eq!(
            evaluate_service(&negative, 60),
            Err(ServiceError::NegativeQuantity)
        );
    }

    #[test]
    fn metadata_digest_floor_and_tolerance_are_validated() {
        let mut bad_digest = service(vec![interval(0, 60, 1.0, 1.0)]);
        bad_digest.source_sha256 = "not-a-digest".to_owned();
        assert_eq!(
            evaluate_service(&bad_digest, 60),
            Err(ServiceError::InvalidSourceDigest)
        );

        let mut bad_floor = service(vec![interval(0, 60, 1.0, 1.0)]);
        bad_floor.minimum_service_fraction = 0.0;
        assert_eq!(
            evaluate_service(&bad_floor, 60),
            Err(ServiceError::InvalidServiceFloor)
        );

        let mut bad_tolerance = service(vec![interval(0, 60, 1.0, 1.0)]);
        bad_tolerance.quantity_tolerance = -0.1;
        assert_eq!(
            evaluate_service(&bad_tolerance, 60),
            Err(ServiceError::InvalidQuantityTolerance)
        );
    }

    #[test]
    fn zero_recovery_hold_is_rejected() {
        let mut input = service(vec![interval(0, 60, 100.0, 100.0)]);
        input.recovery_hold_seconds = 0;
        assert_eq!(
            evaluate_service(&input, 60),
            Err(ServiceError::InvalidRecoveryHold)
        );
    }

    #[test]
    fn tolerance_cannot_consume_the_entire_service_floor() {
        let mut input = service(vec![interval(0, 60, 100.0, 0.0)]);
        input.quantity_tolerance = 80.0;
        assert_eq!(
            evaluate_service(&input, 60),
            Err(ServiceError::InvalidQuantityTolerance)
        );
    }

    #[test]
    fn scenario_rejects_duplicate_service_ids() {
        let first = service(vec![interval(0, 60, 100.0, 100.0)]);
        let second = first.clone();
        assert_eq!(
            evaluate_scenario(&scenario(vec![first, second], 60)),
            Err(ServiceError::DuplicateServiceId)
        );
    }

    #[test]
    fn scenario_returns_independent_native_unit_vectors() {
        let power = service(vec![interval(0, 60, 100.0, 90.0)]);
        let mut water = service(vec![interval(0, 60, 10.0, 5.0)]);
        water.service_id = "hospital-water".to_owned();
        water.quantity_unit = "m3".to_owned();
        water.minimum_service_fraction = 0.4;

        let result = evaluate_scenario(&scenario(vec![power, water], 60)).unwrap();
        assert_eq!(result.services.len(), 2);
        assert_eq!(result.services[0].quantity_unit, "kWh");
        assert_eq!(result.services[0].total_unserved_quantity, 10.0);
        assert_eq!(result.services[1].quantity_unit, "m3");
        assert_eq!(result.services[1].total_unserved_quantity, 5.0);
    }

    #[test]
    fn zero_total_demand_is_not_a_valid_service_benchmark() {
        let input = service(vec![interval(0, 60, 0.0, 0.0)]);
        assert_eq!(
            evaluate_service(&input, 60),
            Err(ServiceError::NoPositiveDemand)
        );
    }

    #[test]
    fn tolerance_is_applied_in_the_declared_native_unit() {
        let mut input = service(vec![interval(0, 60, 100.0, 79.5)]);
        input.quantity_tolerance = 0.5;
        let result = evaluate_service(&input, 60).unwrap();

        assert_eq!(result.floor_violation_episode_count, 0);
        assert!(result.initially_above_service_floor);
        assert_eq!(result.floor_recovery, None);
        assert_eq!(result.demand_weighted_served_fraction, 0.795);
        assert_eq!(result.total_unserved_quantity, 20.5);
    }
}
