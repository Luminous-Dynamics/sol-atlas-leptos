#![forbid(unsafe_code)]
//! Compute-budgeted temporal-mesh allocation for rolling-horizon energy dispatch.
//!
//! The allocator refines safety-critical windows first, then spends remaining
//! interval budget on the highest marginal lower-bound value of information.
//! It is a mesh-design benchmark, not an optimizer or an actuator.

use std::collections::BTreeSet;
use std::fmt;

pub const HORIZON_MINUTES: u16 = 1_440;
pub const COARSE_STEP_MINUTES: u16 = 60;
pub const FINE_STEP_MINUTES: u16 = 15;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalRisk {
    pub id: String,
    pub start_minute: u16,
    pub end_minute: u16,
    /// Conservative lower bound on avoidable loss if the event is represented
    /// at fine resolution. This is a synthetic benchmark unit, not currency or kWh.
    pub avoidable_loss_lower_bound_units: u64,
    /// Safety windows are mandatory refinements, never optional VoI candidates.
    pub safety_critical: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshStrategy {
    UniformCoarse,
    UniformFine,
    SafetyFirstValueOfInformation,
}

impl MeshStrategy {
    pub const ALL: [Self; 3] = [
        Self::UniformCoarse,
        Self::UniformFine,
        Self::SafetyFirstValueOfInformation,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::UniformCoarse => "uniform-60m",
            Self::UniformFine => "uniform-15m",
            Self::SafetyFirstValueOfInformation => "safety-first-voi",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeshCell {
    pub start_minute: u16,
    pub end_minute: u16,
    pub resolution_minutes: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshPlan {
    pub strategy: MeshStrategy,
    pub cells: Vec<MeshCell>,
    pub compute_cost_units: u32,
    pub compute_budget_units: u32,
    pub budget_remaining_units: i64,
    pub safety_resolution_violations: u32,
    pub unresolved_loss_units: u64,
    pub refined_event_ids: Vec<String>,
}

impl MeshPlan {
    pub fn within_budget(&self) -> bool {
        self.compute_cost_units <= self.compute_budget_units
    }

    pub fn accepted(&self) -> bool {
        self.within_budget() && self.safety_resolution_violations == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshError {
    EmptyEventId,
    DuplicateEventId,
    InvalidEventWindow,
    InvalidMeshCertificate,
    SafetyBudgetInsufficient {
        minimum_required: u32,
        available: u32,
    },
}

impl fmt::Display for MeshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEventId => formatter.write_str("event identifier must not be empty"),
            Self::DuplicateEventId => formatter.write_str("event identifiers must be unique"),
            Self::InvalidEventWindow => {
                formatter.write_str("event window must be non-empty and inside the horizon")
            }
            Self::SafetyBudgetInsufficient {
                minimum_required,
                available,
            } => write!(
                formatter,
                "safety refinement requires {minimum_required} compute units but only {available} are available"
            ),
        }
    }
}

impl std::error::Error for MeshError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Segment {
    start_minute: u16,
    end_minute: u16,
}

impl Segment {
    fn duration(self) -> u16 {
        self.end_minute - self.start_minute
    }

    fn interval_count(self, resolution_minutes: u16) -> u32 {
        u32::from(self.duration()).div_ceil(u32::from(resolution_minutes))
    }

    fn is_inside(self, event: &TemporalRisk) -> bool {
        self.start_minute >= event.start_minute && self.end_minute <= event.end_minute
    }
}

fn validate_events(events: &[TemporalRisk]) -> Result<(), MeshError> {
    let mut identifiers = BTreeSet::new();
    for event in events {
        if event.id.trim().is_empty() {
            return Err(MeshError::EmptyEventId);
        }
        if !identifiers.insert(event.id.as_str()) {
            return Err(MeshError::DuplicateEventId);
        }
        if event.start_minute >= event.end_minute || event.end_minute > HORIZON_MINUTES {
            return Err(MeshError::InvalidEventWindow);
        }
    }
    Ok(())
}

fn partition_horizon(events: &[TemporalRisk]) -> Vec<Segment> {
    let mut boundaries = BTreeSet::from([0, HORIZON_MINUTES]);
    for event in events {
        boundaries.insert(event.start_minute);
        boundaries.insert(event.end_minute);
    }
    let sorted: Vec<u16> = boundaries.into_iter().collect();
    sorted
        .windows(2)
        .map(|pair| Segment {
            start_minute: pair[0],
            end_minute: pair[1],
        })
        .collect()
}

fn segment_cost(segments: &[Segment], fine: &[bool]) -> u32 {
    segments
        .iter()
        .zip(fine)
        .map(|(segment, is_fine)| {
            segment.interval_count(if *is_fine {
                FINE_STEP_MINUTES
            } else {
                COARSE_STEP_MINUTES
            })
        })
        .sum()
}

fn event_segment_indices(segments: &[Segment], event: &TemporalRisk) -> Vec<usize> {
    segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| segment.is_inside(event).then_some(index))
        .collect()
}

fn additional_cost(segments: &[Segment], fine: &[bool], event: &TemporalRisk) -> u32 {
    event_segment_indices(segments, event)
        .into_iter()
        .filter(|index| !fine[*index])
        .map(|index| {
            segments[index].interval_count(FINE_STEP_MINUTES)
                - segments[index].interval_count(COARSE_STEP_MINUTES)
        })
        .sum()
}

fn mark_event_fine(segments: &[Segment], fine: &mut [bool], event: &TemporalRisk) {
    for index in event_segment_indices(segments, event) {
        fine[index] = true;
    }
}

fn build_cells(segments: &[Segment], fine: &[bool]) -> Vec<MeshCell> {
    let mut cells = Vec::new();
    for (segment, is_fine) in segments.iter().zip(fine) {
        let resolution = if *is_fine {
            FINE_STEP_MINUTES
        } else {
            COARSE_STEP_MINUTES
        };
        let mut start = segment.start_minute;
        while start < segment.end_minute {
            let end = start.saturating_add(resolution).min(segment.end_minute);
            cells.push(MeshCell {
                start_minute: start,
                end_minute: end,
                resolution_minutes: resolution,
            });
            start = end;
        }
    }
    cells
}

fn safety_violations(cells: &[MeshCell], events: &[TemporalRisk]) -> u32 {
    u32::try_from(
        events
            .iter()
            .filter(|event| event.safety_critical)
            .map(|event| {
                cells
                    .iter()
                    .filter(|cell| {
                        cell.start_minute < event.end_minute
                            && cell.end_minute > event.start_minute
                            && cell.resolution_minutes > FINE_STEP_MINUTES
                    })
                    .count()
            })
            .sum::<usize>(),
    )
    .unwrap_or(u32::MAX)
}

fn unresolved_loss(events: &[TemporalRisk], cells: &[MeshCell]) -> u64 {
    let total = events.iter().fold(0u128, |total, event| {
        let coarse_minutes: u64 = cells
            .iter()
            .filter(|cell| {
                cell.start_minute < event.end_minute
                    && cell.end_minute > event.start_minute
                    && cell.resolution_minutes > FINE_STEP_MINUTES
            })
            .map(|cell| {
                u64::from(
                    cell.end_minute.min(event.end_minute)
                        - cell.start_minute.max(event.start_minute),
                )
            })
            .sum();
        let event_duration = u128::from(event.end_minute - event.start_minute);
        let event_loss = u128::from(event.avoidable_loss_lower_bound_units)
            .saturating_mul(u128::from(coarse_minutes))
            .div_ceil(event_duration);
        total.saturating_add(event_loss)
    });
    u64::try_from(total).unwrap_or(u64::MAX)
}

fn compare_value_density(
    left: &TemporalRisk,
    left_cost: u32,
    right: &TemporalRisk,
    right_cost: u32,
) -> std::cmp::Ordering {
    // u64 values multiplied by u32 marginal costs require up to 96 bits.
    // Compare in u128 to preserve ordering instead of saturating both scores.
    let left_score =
        u128::from(left.avoidable_loss_lower_bound_units) * u128::from(right_cost.max(1));
    let right_score =
        u128::from(right.avoidable_loss_lower_bound_units) * u128::from(left_cost.max(1));
    right_score
        .cmp(&left_score)
        .then_with(|| left.id.cmp(&right.id))
}

/// Plan a 24-hour temporal mesh under an interval-count compute budget.
///
/// All event start/end boundaries are preserved exactly. Safety-critical event
/// windows are refined to 15 minutes before optional events are considered.
/// Remaining budget is allocated greedily by lower-bound avoided loss per
/// incremental interval. The denominator is marginal interval count, not a
/// claimed wall-clock runtime; calibration against actual solver timings is a
/// separate benchmark stage.
pub fn plan_mesh(
    events: &[TemporalRisk],
    strategy: MeshStrategy,
    compute_budget_units: u32,
) -> Result<MeshPlan, MeshError> {
    validate_events(events)?;
    let segments = partition_horizon(events);
    let mut fine = vec![false; segments.len()];

    match strategy {
        MeshStrategy::UniformCoarse => {}
        MeshStrategy::UniformFine => fine.fill(true),
        MeshStrategy::SafetyFirstValueOfInformation => {
            for event in events.iter().filter(|event| event.safety_critical) {
                mark_event_fine(&segments, &mut fine, event);
            }
            let safety_cost = segment_cost(&segments, &fine);
            if safety_cost > compute_budget_units {
                return Err(MeshError::SafetyBudgetInsufficient {
                    minimum_required: safety_cost,
                    available: compute_budget_units,
                });
            }

            let mut candidates: Vec<&TemporalRisk> = events
                .iter()
                .filter(|event| !event.safety_critical)
                .collect();

            let mut current_cost = safety_cost;
            loop {
                // Recompute marginal cost after each refinement. Overlapping
                // event windows can share already-refined segments, changing
                // their value-per-incremental-cell ranking.
                let mut ranked: Vec<(&TemporalRisk, u32)> = candidates
                    .iter()
                    .filter_map(|event| {
                        let marginal_cost = additional_cost(&segments, &fine, event);
                        (marginal_cost > 0
                            && current_cost.saturating_add(marginal_cost) <= compute_budget_units)
                            .then_some((*event, marginal_cost))
                    })
                    .collect();
                ranked.sort_by(|(left, left_cost), (right, right_cost)| {
                    compare_value_density(left, *left_cost, right, *right_cost)
                });

                let Some((selected, marginal_cost)) = ranked.first().copied() else {
                    break;
                };
                mark_event_fine(&segments, &mut fine, selected);
                current_cost += marginal_cost;
                candidates.retain(|candidate| candidate.id != selected.id);
            }
        }
    }

    let cells = build_cells(&segments, &fine);
    let compute_cost_units = u32::try_from(cells.len()).unwrap_or(u32::MAX);
    let mut refined_event_ids: Vec<String> = events
        .iter()
        .filter(|event| {
            event_segment_indices(&segments, event)
                .iter()
                .all(|index| fine[*index])
        })
        .map(|event| event.id.clone())
        .collect();
    refined_event_ids.sort();

    let plan = MeshPlan {
        strategy,
        compute_cost_units,
        compute_budget_units,
        budget_remaining_units: i64::from(compute_budget_units) - i64::from(compute_cost_units),
        safety_resolution_violations: safety_violations(&cells, events),
        unresolved_loss_units: unresolved_loss(events, &cells),
        cells,
        refined_event_ids,
    };
    verify_mesh(events, &plan)?;
    Ok(plan)
}

/// Independently check coverage, chronology, mandatory boundaries, resolution
/// declarations and the metric fields emitted by the mesh planner.
pub fn verify_mesh(events: &[TemporalRisk], plan: &MeshPlan) -> Result<(), MeshError> {
    validate_events(events)?;
    if plan.cells.is_empty()
        || plan.cells[0].start_minute != 0
        || plan.cells.last().map(|cell| cell.end_minute) != Some(HORIZON_MINUTES)
    {
        return Err(MeshError::InvalidEventWindow);
    }

    for (index, cell) in plan.cells.iter().enumerate() {
        let strategy_resolution_mismatch = match plan.strategy {
            MeshStrategy::UniformCoarse => cell.resolution_minutes != COARSE_STEP_MINUTES,
            MeshStrategy::UniformFine => cell.resolution_minutes != FINE_STEP_MINUTES,
            MeshStrategy::SafetyFirstValueOfInformation => false,
        };
        if cell.start_minute >= cell.end_minute
            || cell.end_minute - cell.start_minute > cell.resolution_minutes
            || ![FINE_STEP_MINUTES, COARSE_STEP_MINUTES].contains(&cell.resolution_minutes)
            || strategy_resolution_mismatch
        {
            return Err(MeshError::InvalidMeshCertificate);
        }
        if index > 0 && plan.cells[index - 1].end_minute != cell.start_minute {
            return Err(MeshError::InvalidEventWindow);
        }
        for event in events {
            if (cell.start_minute < event.start_minute && event.start_minute < cell.end_minute)
                || (cell.start_minute < event.end_minute && event.end_minute < cell.end_minute)
            {
                return Err(MeshError::InvalidEventWindow);
            }
        }
    }

    if plan.strategy == MeshStrategy::SafetyFirstValueOfInformation
        && safety_violations(&plan.cells, events) > 0
    {
        return Err(MeshError::InvalidMeshCertificate);
    }

    let mut expected_refined_event_ids: Vec<String> = events
        .iter()
        .filter(|event| {
            plan.cells
                .iter()
                .filter(|cell| {
                    cell.start_minute < event.end_minute && cell.end_minute > event.start_minute
                })
                .all(|cell| cell.resolution_minutes == FINE_STEP_MINUTES)
        })
        .map(|event| event.id.clone())
        .collect();
    expected_refined_event_ids.sort();

    let measured_cost = u32::try_from(plan.cells.len()).unwrap_or(u32::MAX);
    if measured_cost != plan.compute_cost_units
        || plan.budget_remaining_units
            != i64::from(plan.compute_budget_units) - i64::from(measured_cost)
        || plan.safety_resolution_violations != safety_violations(&plan.cells, events)
        || plan.unresolved_loss_units != unresolved_loss(events, &plan.cells)
        || plan.refined_event_ids != expected_refined_event_ids
    {
        return Err(MeshError::InvalidMeshCertificate);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshScenario {
    pub id: String,
    pub events: Vec<TemporalRisk>,
    pub compute_budget_units: u32,
}

fn make_scenario(kind: usize, variant: u32) -> MeshScenario {
    let scale = u64::from(variant + 1);
    let mut events = Vec::new();
    match kind {
        0 => events.push(TemporalRisk {
            id: "evening-peak".to_owned(),
            start_minute: 1_020,
            end_minute: 1_200,
            avoidable_loss_lower_bound_units: 1_000 * scale,
            safety_critical: false,
        }),
        1 => events.push(TemporalRisk {
            id: "solar-ramp".to_owned(),
            start_minute: 420,
            end_minute: 600,
            avoidable_loss_lower_bound_units: 2_400 * scale,
            safety_critical: false,
        }),
        2 => events.push(TemporalRisk {
            id: "storm-outage".to_owned(),
            start_minute: 840,
            end_minute: 960,
            avoidable_loss_lower_bound_units: 6_000 * scale,
            safety_critical: true,
        }),
        3 => events.push(TemporalRisk {
            id: "telemetry-recovery".to_owned(),
            start_minute: 360,
            end_minute: 420,
            avoidable_loss_lower_bound_units: 4_000 * scale,
            safety_critical: true,
        }),
        4 => events.push(TemporalRisk {
            id: "equipment-derate".to_owned(),
            start_minute: 720,
            end_minute: 900,
            avoidable_loss_lower_bound_units: 5_000 * scale,
            safety_critical: true,
        }),
        5 => events.push(TemporalRisk {
            id: "price-spike".to_owned(),
            start_minute: 960,
            end_minute: 1_080,
            avoidable_loss_lower_bound_units: 3_000 * scale,
            safety_critical: false,
        }),
        6 => events.push(TemporalRisk {
            id: "load-pv-mismatch".to_owned(),
            start_minute: 480,
            end_minute: 660,
            avoidable_loss_lower_bound_units: 3_600 * scale,
            safety_critical: false,
        }),
        _ => {
            events.push(TemporalRisk {
                id: "morning-ramp".to_owned(),
                start_minute: 360,
                end_minute: 480,
                avoidable_loss_lower_bound_units: 5_000 * scale,
                safety_critical: false,
            });
            events.push(TemporalRisk {
                id: "storm-outage".to_owned(),
                start_minute: 840,
                end_minute: 960,
                avoidable_loss_lower_bound_units: 6_000 * scale,
                safety_critical: true,
            });
            events.push(TemporalRisk {
                id: "evening-peak".to_owned(),
                start_minute: 1_020,
                end_minute: 1_200,
                avoidable_loss_lower_bound_units: 6_000 * scale,
                safety_critical: false,
            });
        }
    }
    let compute_budget_units = if kind == 7 {
        match variant {
            0 => 30,
            1 => 33,
            2 => 36,
            _ => 45,
        }
    } else {
        48 + variant * 3
    };

    MeshScenario {
        id: format!("mesh-scenario-{kind:02}-{variant:02}"),
        events,
        compute_budget_units,
    }
}

/// Deterministic 32-case panel: eight operating regimes and four value scales.
pub fn frozen_scenarios() -> Vec<MeshScenario> {
    let mut scenarios = Vec::with_capacity(32);
    for kind in 0..8 {
        for variant in 0..4 {
            scenarios.push(make_scenario(kind, variant));
        }
    }
    scenarios
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(
        id: &str,
        start_minute: u16,
        end_minute: u16,
        value: u64,
        safety_critical: bool,
    ) -> TemporalRisk {
        TemporalRisk {
            id: id.to_owned(),
            start_minute,
            end_minute,
            avoidable_loss_lower_bound_units: value,
            safety_critical,
        }
    }

    #[test]
    fn every_mesh_covers_the_full_horizon_without_gaps_or_overlaps() {
        let events = vec![
            event("safety", 420, 540, 7_000, true),
            event("value", 900, 1_080, 4_000, false),
        ];
        for strategy in MeshStrategy::ALL {
            let plan = plan_mesh(&events, strategy, 128).unwrap();
            verify_mesh(&events, &plan).unwrap();
            assert_eq!(plan.cells.first().unwrap().start_minute, 0);
            assert_eq!(plan.cells.last().unwrap().end_minute, HORIZON_MINUTES);
        }
    }

    #[test]
    fn all_event_boundaries_are_preserved_exactly() {
        let events = vec![
            event("a", 113, 287, 1_000, false),
            event("b", 777, 901, 2_000, true),
        ];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 80).unwrap();
        for boundary in [113, 287, 777, 901] {
            assert!(plan.cells.iter().any(|cell| cell.start_minute == boundary));
            assert!(plan.cells.iter().any(|cell| cell.end_minute == boundary));
        }
        assert!(
            plan.cells
                .iter()
                .all(|cell| !(cell.start_minute < 777 && cell.end_minute > 777))
        );
        assert!(
            plan.cells
                .iter()
                .all(|cell| !(cell.start_minute < 901 && cell.end_minute > 901))
        );
    }

    #[test]
    fn safety_windows_are_refined_before_any_optional_value_window() {
        let events = vec![
            event("optional", 180, 300, 10_000, false),
            event("safety", 840, 960, 1, true),
        ];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 32).unwrap();
        assert!(plan.refined_event_ids.contains(&"safety".to_owned()));
        assert!(!plan.refined_event_ids.contains(&"optional".to_owned()));
        assert_eq!(plan.safety_resolution_violations, 0);
        assert!(plan.accepted());
    }

    #[test]
    fn the_greedy_value_allocator_selects_the_highest_marginal_value_window() {
        let events = vec![
            event("low", 180, 240, 2_000, false),
            event("high", 900, 960, 9_000, false),
        ];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 28).unwrap();
        assert!(plan.refined_event_ids.contains(&"high".to_owned()));
        assert!(!plan.refined_event_ids.contains(&"low".to_owned()));
    }

    #[test]
    fn overlapping_optional_windows_recompute_marginal_value_after_each_refinement() {
        let events = vec![
            event("first", 180, 300, 10_000, false),
            event("overlap", 240, 360, 4_000, false),
            event("separate", 360, 420, 3_000, false),
        ];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 33).unwrap();

        // First consumes six extra cells. The overlap then costs only three
        // marginal cells and outranks the separate lower-density event.
        assert!(plan.refined_event_ids.contains(&"first".to_owned()));
        assert!(plan.refined_event_ids.contains(&"overlap".to_owned()));
        assert!(!plan.refined_event_ids.contains(&"separate".to_owned()));
        assert_eq!(plan.compute_cost_units, 33);
    }

    #[test]
    fn insufficient_compute_for_mandatory_safety_refinement_fails_closed() {
        let events = vec![event("safety", 840, 960, 5_000, true)];
        let error =
            plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 25).unwrap_err();
        assert_eq!(
            error,
            MeshError::SafetyBudgetInsufficient {
                minimum_required: 30,
                available: 25,
            }
        );
    }

    #[test]
    fn uniform_fine_baseline_reports_budget_exhaustion_instead_of_claiming_feasibility() {
        let events = vec![event("peak", 900, 1_080, 2_000, false)];
        let plan = plan_mesh(&events, MeshStrategy::UniformFine, 60).unwrap();
        assert_eq!(plan.compute_cost_units, 96);
        assert!(!plan.within_budget());
        assert!(!plan.accepted());
    }

    #[test]
    fn uniform_coarse_baseline_exposes_safety_resolution_violations() {
        let events = vec![event("outage", 840, 960, 4_000, true)];
        let plan = plan_mesh(&events, MeshStrategy::UniformCoarse, 64).unwrap();
        assert!(plan.safety_resolution_violations > 0);
        assert!(!plan.accepted());
    }

    #[test]
    fn overlapping_event_windows_do_not_create_gaps_or_duplicate_cells() {
        let events = vec![
            event("safety", 420, 600, 4_000, true),
            event("value", 540, 720, 8_000, false),
        ];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 64).unwrap();
        verify_mesh(&events, &plan).unwrap();
        assert_eq!(plan.cells.first().unwrap().start_minute, 0);
        assert_eq!(plan.cells.last().unwrap().end_minute, HORIZON_MINUTES);
        assert!(plan.refined_event_ids.contains(&"safety".to_owned()));
    }

    #[test]
    fn scenario_order_does_not_change_the_selected_mesh() {
        let mut events = vec![
            event("b", 180, 300, 4_000, false),
            event("a", 840, 960, 6_000, true),
            event("c", 1_020, 1_200, 3_000, false),
        ];
        let first = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 56).unwrap();
        events.reverse();
        let second = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 56).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn duplicate_ids_and_invalid_event_windows_are_rejected() {
        let duplicate = vec![
            event("same", 100, 200, 1_000, false),
            event("same", 300, 400, 2_000, false),
        ];
        assert_eq!(
            plan_mesh(&duplicate, MeshStrategy::UniformCoarse, 64),
            Err(MeshError::DuplicateEventId)
        );

        let invalid = vec![event("bad", 1_000, 900, 1_000, false)];
        assert_eq!(
            plan_mesh(&invalid, MeshStrategy::UniformCoarse, 64),
            Err(MeshError::InvalidEventWindow)
        );
    }

    #[test]
    fn frozen_panel_has_32_reproducible_scenarios() {
        let first = frozen_scenarios();
        let second = frozen_scenarios();
        assert_eq!(first, second);
        assert_eq!(first.len(), 32);
    }

    #[test]
    fn value_density_ordering_remains_exact_near_u64_maximum() {
        let events = vec![
            event("a-lower-density", 180, 240, u64::MAX - 1, false),
            event("z-higher-density", 900, 960, u64::MAX, false),
        ];
        let plan = plan_mesh(
            &events,
            MeshStrategy::SafetyFirstValueOfInformation,
            27,
        )
        .unwrap();

        assert!(plan
            .refined_event_ids
            .contains(&"z-higher-density".to_owned()));
        assert!(!plan
            .refined_event_ids
            .contains(&"a-lower-density".to_owned()));
    }

    #[test]
    fn verifier_rejects_tampered_refined_event_metadata() {
        let events = vec![event("safety", 840, 960, 8_000, true)];
        let mut plan = plan_mesh(
            &events,
            MeshStrategy::SafetyFirstValueOfInformation,
            40,
        )
        .unwrap();
        plan.refined_event_ids.clear();

        assert_eq!(
            verify_mesh(&events, &plan),
            Err(MeshError::InvalidMeshCertificate)
        );
    }

    #[test]
    fn verifier_rejects_a_fine_plan_relabelled_as_uniform_coarse() {
        let events = vec![event("optional", 840, 960, 8_000, false)];
        let mut plan = plan_mesh(&events, MeshStrategy::UniformFine, 100).unwrap();
        plan.strategy = MeshStrategy::UniformCoarse;

        assert_eq!(
            verify_mesh(&events, &plan),
            Err(MeshError::InvalidMeshCertificate)
        );
    }

    #[test]
    fn verifier_rejects_safety_window_coarsening_even_when_report_counters_are_recomputed() {
        let events = vec![event("safety", 840, 960, 8_000, true)];
        let mut plan = plan_mesh(
            &events,
            MeshStrategy::SafetyFirstValueOfInformation,
            40,
        )
        .unwrap();
        for cell in plan.cells.iter_mut().filter(|cell| {
            cell.start_minute >= 840 && cell.end_minute <= 960
        }) {
            cell.resolution_minutes = COARSE_STEP_MINUTES;
        }
        plan.compute_cost_units =
            u32::try_from(plan.cells.len()).unwrap_or(u32::MAX);
        plan.budget_remaining_units =
            i64::from(plan.compute_budget_units) - i64::from(plan.compute_cost_units);
        plan.safety_resolution_violations = safety_violations(&plan.cells, &events);
        plan.unresolved_loss_units = unresolved_loss(&events, &plan.cells);

        assert_eq!(
            verify_mesh(&events, &plan),
            Err(MeshError::InvalidMeshCertificate)
        );
    }

    #[test]
    fn compound_scenario_budgets_force_partial_optional_refinement() {
        let scenarios = frozen_scenarios();
        let compound: Vec<&MeshScenario> = scenarios
            .iter()
            .filter(|scenario| scenario.id.starts_with("mesh-scenario-07-"))
            .collect();
        assert_eq!(compound.len(), 4);

        let expected = [
            (30, false, false),
            (33, false, false),
            (36, true, false),
            (45, true, true),
        ];
        for (scenario, (budget, morning, evening)) in compound.iter().zip(expected) {
            assert_eq!(scenario.compute_budget_units, budget);
            let plan = plan_mesh(
                &scenario.events,
                MeshStrategy::SafetyFirstValueOfInformation,
                budget,
            )
            .unwrap();

            assert!(plan.refined_event_ids.contains(&"storm-outage".to_owned()));
            assert_eq!(
                plan.refined_event_ids.contains(&"morning-ramp".to_owned()),
                morning
            );
            assert_eq!(
                plan.refined_event_ids.contains(&"evening-peak".to_owned()),
                evening
            );
            assert!(plan.accepted());
        }
    }

    #[test]
    fn every_safety_case_is_either_refined_or_explicitly_budget_infeasible() {
        for scenario in frozen_scenarios() {
            let outcome = plan_mesh(
                &scenario.events,
                MeshStrategy::SafetyFirstValueOfInformation,
                scenario.compute_budget_units,
            );
            match outcome {
                Ok(plan) => {
                    assert!(plan.accepted(), "{}", scenario.id);
                    assert!(plan.within_budget(), "{}", scenario.id);
                    assert_eq!(plan.safety_resolution_violations, 0);
                }
                Err(MeshError::SafetyBudgetInsufficient { .. }) => {}
                Err(error) => panic!("{}: unexpected error: {error}", scenario.id),
            }
        }
    }

    #[test]
    fn residual_loss_is_zero_when_every_event_window_is_refined() {
        let events = vec![event("safety", 840, 960, 8_000, true)];
        let plan = plan_mesh(&events, MeshStrategy::SafetyFirstValueOfInformation, 40).unwrap();
        assert_eq!(plan.unresolved_loss_units, 0);
    }
}
