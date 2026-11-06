// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exhaustive, bounded sensitivity analysis for ambiguous architectural
//! membership. This module enumerates every per-level candidate assignment
//! when the Cartesian product fits the declared work limits. It never samples,
//! imputes probabilities, or silently truncates alternatives.
//!
//! The first version is deliberately conservative: candidate choices are
//! combined as an unconstrained Cartesian product. It is an outer sensitivity
//! envelope, not proof that every joint assignment is graph-compatible.
//! HDB-17 remains the authority for architecture-graph validity.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::heritage_morphology::{
    ArchitectureGroupLevel, GroupMembership, HeritageMorphologyInput, HeritageMorphologyError,
    IntervalDistributionSummary, MeasurementInterval, MembershipStatus,
    StoneMorphologyObservation, aggregate_architecture_morphology,
};

pub const HERITAGE_MEMBERSHIP_SENSITIVITY_SCHEMA_VERSION: &str =
    "luminous.heritage-membership-sensitivity.v0.1";
pub const MAX_MEMBERSHIP_SENSITIVITY_SCENARIOS: usize = 4_096;
pub const MAX_MEMBERSHIP_SENSITIVITY_WORK_UNITS: usize = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MembershipSensitivityLimits {
    /// Maximum number of complete candidate-assignment scenarios to enumerate.
    pub maximum_scenarios: usize,
    /// Maximum value of scenario_count * (stone_count + possible_group_count).
    pub maximum_work_units: usize,
}

impl Default for MembershipSensitivityLimits {
    fn default() -> Self {
        Self {
            maximum_scenarios: MAX_MEMBERSHIP_SENSITIVITY_SCENARIOS,
            maximum_work_units: MAX_MEMBERSHIP_SENSITIVITY_WORK_UNITS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarRange {
    pub minimum: f64,
    pub maximum: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipMetricEnvelope {
    /// Outer interval across all enumerated assignments and source measurement
    /// bounds. This is not a confidence or credible interval.
    pub enclosing_interval: MeasurementInterval,
    /// Range of the summary's lower endpoint across assignment scenarios.
    pub lower_endpoint_range: ScalarRange,
    /// Range of the summary's upper endpoint across assignment scenarios.
    pub upper_endpoint_range: ScalarRange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSummarySensitivity {
    pub mean: MembershipMetricEnvelope,
    pub p10: MembershipMetricEnvelope,
    pub median: MembershipMetricEnvelope,
    pub p90: MembershipMetricEnvelope,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupMembershipSensitivitySummary {
    pub level: ArchitectureGroupLevel,
    pub group_id: String,
    /// Includes zero in scenarios where this group receives no assigned stones.
    pub minimum_assigned_stone_count: usize,
    pub maximum_assigned_stone_count: usize,
    /// Number of scenarios in which at least one stone is assigned to this group.
    pub non_empty_scenario_count: usize,
    pub plan_footprint_area_m2: Option<MetricSummarySensitivity>,
    pub mrr_area_ratio: Option<MetricSummarySensitivity>,
    pub elongation_ratio: Option<MetricSummarySensitivity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageMembershipSensitivityReport {
    pub schema_version: String,
    pub sensitivity_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub dataset_sha256: String,
    pub architecture_graph_sha256: String,
    pub measurement_protocol_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_sha256: String,
    pub target_level: ArchitectureGroupLevel,
    pub physical_stone_count: usize,
    pub fixed_assigned_stone_count: usize,
    pub ambiguous_stone_count: usize,
    pub unassigned_stone_count: usize,
    pub not_assessed_stone_count: usize,
    pub scenario_count: usize,
    pub maximum_scenarios: usize,
    pub preflight_work_units: usize,
    pub maximum_preflight_work_units: usize,
    pub groups: Vec<GroupMembershipSensitivitySummary>,
    pub scenario_semantics: String,
    pub interval_interpretation: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageMembershipSensitivityError {
    InvalidInput(HeritageMorphologyError),
    EmptySensitivityId,
    InvalidLimits,
    ScenarioCountOverflow,
    ScenarioLimitExceeded { required: usize, maximum: usize },
    WorkEstimateOverflow,
    WorkLimitExceeded { required: usize, maximum: usize },
}

impl fmt::Display for HeritageMembershipSensitivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage membership sensitivity error: {self:?}")
    }
}

impl std::error::Error for HeritageMembershipSensitivityError {}

#[derive(Debug, Clone, Default)]
struct MetricEnvelopeAccumulator {
    mean: Option<MembershipMetricEnvelope>,
    p10: Option<MembershipMetricEnvelope>,
    median: Option<MembershipMetricEnvelope>,
    p90: Option<MembershipMetricEnvelope>,
}

impl MetricEnvelopeAccumulator {
    fn observe(&mut self, summary: IntervalDistributionSummary) {
        extend_envelope(&mut self.mean, summary.mean);
        extend_envelope(&mut self.p10, summary.p10);
        extend_envelope(&mut self.median, summary.median);
        extend_envelope(&mut self.p90, summary.p90);
    }

    fn finish(self) -> Option<MetricSummarySensitivity> {
        Some(MetricSummarySensitivity {
            mean: self.mean?,
            p10: self.p10?,
            median: self.median?,
            p90: self.p90?,
        })
    }
}

fn extend_envelope(
    current: &mut Option<MembershipMetricEnvelope>,
    interval: MeasurementInterval,
) {
    match current {
        Some(envelope) => {
            envelope.lower_endpoint_range.minimum =
                envelope.lower_endpoint_range.minimum.min(interval.lower);
            envelope.lower_endpoint_range.maximum =
                envelope.lower_endpoint_range.maximum.max(interval.lower);
            envelope.upper_endpoint_range.minimum =
                envelope.upper_endpoint_range.minimum.min(interval.upper);
            envelope.upper_endpoint_range.maximum =
                envelope.upper_endpoint_range.maximum.max(interval.upper);
            envelope.enclosing_interval.lower = envelope
                .enclosing_interval
                .lower
                .min(interval.lower);
            envelope.enclosing_interval.upper = envelope
                .enclosing_interval
                .upper
                .max(interval.upper);
        }
        None => {
            *current = Some(MembershipMetricEnvelope {
                enclosing_interval: interval,
                lower_endpoint_range: ScalarRange {
                    minimum: interval.lower,
                    maximum: interval.lower,
                },
                upper_endpoint_range: ScalarRange {
                    minimum: interval.upper,
                    maximum: interval.upper,
                },
            });
        }
    }
}

#[derive(Debug)]
struct GroupSensitivityAccumulator {
    minimum_assigned_stone_count: usize,
    maximum_assigned_stone_count: usize,
    non_empty_scenario_count: usize,
    area: MetricEnvelopeAccumulator,
    regularity: MetricEnvelopeAccumulator,
    elongation: MetricEnvelopeAccumulator,
}

impl Default for GroupSensitivityAccumulator {
    fn default() -> Self {
        Self {
            minimum_assigned_stone_count: usize::MAX,
            maximum_assigned_stone_count: 0,
            non_empty_scenario_count: 0,
            area: MetricEnvelopeAccumulator::default(),
            regularity: MetricEnvelopeAccumulator::default(),
            elongation: MetricEnvelopeAccumulator::default(),
        }
    }
}

impl GroupSensitivityAccumulator {
    fn observe(&mut self, stones: &[&StoneMorphologyObservation]) {
        let count = stones.len();
        self.minimum_assigned_stone_count =
            self.minimum_assigned_stone_count.min(count);
        self.maximum_assigned_stone_count =
            self.maximum_assigned_stone_count.max(count);
        if count == 0 {
            return;
        }
        self.non_empty_scenario_count += 1;

        if let Some(summary) = summarize_intervals_safely(
            stones
                .iter()
                .map(|stone| stone.plan_footprint_area_m2)
                .collect(),
        ) {
            self.area.observe(summary);
        }
        if let Some(summary) = summarize_intervals_safely(
            stones
                .iter()
                .filter_map(|stone| stone.mrr_area_ratio)
                .collect(),
        ) {
            self.regularity.observe(summary);
        }
        if let Some(summary) = summarize_intervals_safely(
            stones
                .iter()
                .filter_map(|stone| stone.elongation_ratio)
                .collect(),
        ) {
            self.elongation.observe(summary);
        }
    }

    fn finish(
        self,
        level: ArchitectureGroupLevel,
        group_id: String,
    ) -> GroupMembershipSensitivitySummary {
        GroupMembershipSensitivitySummary {
            level,
            group_id,
            minimum_assigned_stone_count: self.minimum_assigned_stone_count,
            maximum_assigned_stone_count: self.maximum_assigned_stone_count,
            non_empty_scenario_count: self.non_empty_scenario_count,
            plan_footprint_area_m2: self.area.finish(),
            mrr_area_ratio: self.regularity.finish(),
            elongation_ratio: self.elongation.finish(),
        }
    }
}

fn ceil_log2(value: usize) -> usize {
    if value <= 1 {
        0
    } else {
        usize::BITS as usize - (value - 1).leading_zeros() as usize
    }
}

fn stable_mean(values: impl Iterator<Item = f64>) -> f64 {
    let mut mean = 0.0;
    for (index, value) in values.enumerate() {
        // Online mean avoids overflowing a sum of large, finite positive
        // measurement bounds.
        mean += (value - mean) / (index + 1) as f64;
    }
    mean
}

fn summarize_intervals_safely(
    intervals: Vec<MeasurementInterval>,
) -> Option<IntervalDistributionSummary> {
    if intervals.is_empty() {
        return None;
    }

    let count = intervals.len();

    let mut lower = intervals.iter().map(|value| value.lower).collect::<Vec<_>>();
    let mut upper = intervals.iter().map(|value| value.upper).collect::<Vec<_>>();
    lower.sort_by(f64::total_cmp);
    upper.sort_by(f64::total_cmp);

    let nearest_rank = |values: &[f64], numerator: usize, denominator: usize| {
        let rank = (values.len() * numerator).div_ceil(denominator);
        values[rank.saturating_sub(1).min(values.len() - 1)]
    };

    Some(IntervalDistributionSummary {
        sample_count: count,
        mean: MeasurementInterval {
            lower: stable_mean(intervals.iter().map(|value| value.lower)),
            upper: stable_mean(intervals.iter().map(|value| value.upper)),
        },
        p10: MeasurementInterval {
            lower: nearest_rank(&lower, 1, 10),
            upper: nearest_rank(&upper, 1, 10),
        },
        median: MeasurementInterval {
            lower: nearest_rank(&lower, 1, 2),
            upper: nearest_rank(&upper, 1, 2),
        },
        p90: MeasurementInterval {
            lower: nearest_rank(&lower, 9, 10),
            upper: nearest_rank(&upper, 9, 10),
        },
    })
}

fn membership_at_level(
    stone: &StoneMorphologyObservation,
    level: ArchitectureGroupLevel,
) -> &GroupMembership {
    // aggregate_architecture_morphology validates exactly one row per level.
    stone
        .memberships
        .iter()
        .find(|membership| membership.level == level)
        .expect("validated membership level")
}

/// Exhaustively enumerates the Cartesian product of candidate memberships at
/// one hierarchy level. All explicitly assigned memberships remain fixed;
/// unassigned and not-assessed stones remain outside every group. Candidate
/// alternatives are unweighted and are not interpreted as probabilities.
///
/// The Cartesian product is intentionally unconstrained by graph relations in
/// v0.1. The result is therefore a conservative outer sensitivity envelope,
/// not a graph-validated set of jointly feasible parent assignments.
pub fn evaluate_membership_sensitivity(
    input: &HeritageMorphologyInput,
    sensitivity_id: &str,
    target_level: ArchitectureGroupLevel,
    limits: MembershipSensitivityLimits,
) -> Result<HeritageMembershipSensitivityReport, HeritageMembershipSensitivityError> {
    aggregate_architecture_morphology(input)
        .map_err(HeritageMembershipSensitivityError::InvalidInput)?;

    if sensitivity_id.trim().is_empty() {
        return Err(HeritageMembershipSensitivityError::EmptySensitivityId);
    }
    if limits.maximum_scenarios == 0
        || limits.maximum_scenarios > MAX_MEMBERSHIP_SENSITIVITY_SCENARIOS
        || limits.maximum_work_units == 0
        || limits.maximum_work_units > MAX_MEMBERSHIP_SENSITIVITY_WORK_UNITS
    {
        return Err(HeritageMembershipSensitivityError::InvalidLimits);
    }

    let mut stones = input.stones.iter().collect::<Vec<_>>();
    stones.sort_by(|left, right| {
        left.physical_identity_id
            .cmp(&right.physical_identity_id)
    });

    let mut fixed_assignments = Vec::<(&StoneMorphologyObservation, String)>::new();
    let mut ambiguous_assignments =
        Vec::<(&StoneMorphologyObservation, Vec<String>)>::new();
    let mut group_ids = BTreeSet::new();
    let mut unassigned_stone_count = 0usize;
    let mut not_assessed_stone_count = 0usize;

    for stone in &stones {
        let membership = membership_at_level(stone, target_level);
        match membership.status {
            MembershipStatus::Assigned => {
                let group_id = membership
                    .group_id
                    .as_ref()
                    .expect("validated assigned membership")
                    .clone();
                group_ids.insert(group_id.clone());
                fixed_assignments.push((*stone, group_id));
            }
            MembershipStatus::Ambiguous => {
                let mut candidates = membership.candidate_group_ids.clone();
                candidates.sort();
                candidates.dedup();
                for candidate in &candidates {
                    group_ids.insert(candidate.clone());
                }
                ambiguous_assignments.push((*stone, candidates));
            }
            MembershipStatus::Unassigned => unassigned_stone_count += 1,
            MembershipStatus::NotAssessed => not_assessed_stone_count += 1,
        }
    }

    let mut scenario_count = 1usize;
    for (_, candidates) in &ambiguous_assignments {
        scenario_count = scenario_count
            .checked_mul(candidates.len())
            .ok_or(HeritageMembershipSensitivityError::ScenarioCountOverflow)?;
    }
    if scenario_count > limits.maximum_scenarios {
        return Err(
            HeritageMembershipSensitivityError::ScenarioLimitExceeded {
                required: scenario_count,
                maximum: limits.maximum_scenarios,
            },
        );
    }

    // Upper-bound group sizes across all candidate assignments. Each group
    // can receive every fixed member plus every ambiguous stone that lists it.
    let mut maximum_group_sizes = BTreeMap::<String, usize>::new();
    for (_, group_id) in &fixed_assignments {
        *maximum_group_sizes.entry(group_id.clone()).or_default() += 1;
    }
    for (_, candidates) in &ambiguous_assignments {
        for candidate in candidates {
            *maximum_group_sizes.entry(candidate.clone()).or_default() += 1;
        }
    }

    // Preflight proxy includes stone dispatch/map insertion, one visit per
    // possible group, and a conservative six n*ceil(log2(n)) comparison
    // budget for lower/upper endpoint sorts across three descriptors.
    let group_lookup_factor = ceil_log2(group_ids.len().max(1));
    let assignment_factor = 4usize
        .checked_add(group_lookup_factor)
        .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
    let assignment_work = stones
        .len()
        .checked_mul(assignment_factor)
        .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
    let mut sorting_work = 0usize;
    for group_size in maximum_group_sizes.values().copied() {
        let group_sort_work = group_size
            .checked_mul(ceil_log2(group_size))
            .and_then(|value| value.checked_mul(6))
            .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
        sorting_work = sorting_work
            .checked_add(group_sort_work)
            .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
    }
    let per_scenario_work = assignment_work
        .checked_add(group_ids.len())
        .and_then(|value| value.checked_add(sorting_work))
        .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
    let preflight_work_units = per_scenario_work
        .checked_mul(scenario_count)
        .ok_or(HeritageMembershipSensitivityError::WorkEstimateOverflow)?;
    if preflight_work_units > limits.maximum_work_units {
        return Err(HeritageMembershipSensitivityError::WorkLimitExceeded {
            required: preflight_work_units,
            maximum: limits.maximum_work_units,
        });
    }

    let canonical_group_ids = group_ids.into_iter().collect::<Vec<_>>();
    let mut accumulators = canonical_group_ids
        .iter()
        .cloned()
        .map(|group_id| (group_id, GroupSensitivityAccumulator::default()))
        .collect::<BTreeMap<_, _>>();

    for scenario_index in 0..scenario_count {
        let mut assigned_by_group =
            BTreeMap::<String, Vec<&StoneMorphologyObservation>>::new();

        for (stone, group_id) in &fixed_assignments {
            assigned_by_group
                .entry(group_id.clone())
                .or_default()
                .push(*stone);
        }

        let mut mixed_radix_index = scenario_index;
        for (stone, candidates) in &ambiguous_assignments {
            let selected_index = mixed_radix_index % candidates.len();
            mixed_radix_index /= candidates.len();
            assigned_by_group
                .entry(candidates[selected_index].clone())
                .or_default()
                .push(*stone);
        }

        for group_id in &canonical_group_ids {
            let assigned = assigned_by_group
                .get(group_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            accumulators
                .get_mut(group_id)
                .expect("group accumulator initialised")
                .observe(assigned);
        }
    }

    let groups = canonical_group_ids
        .into_iter()
        .map(|group_id| {
            accumulators
                .remove(&group_id)
                .expect("group accumulator exists")
                .finish(target_level, group_id)
        })
        .collect();

    Ok(HeritageMembershipSensitivityReport {
        schema_version: HERITAGE_MEMBERSHIP_SENSITIVITY_SCHEMA_VERSION.to_owned(),
        sensitivity_id: sensitivity_id.to_owned(),
        site_id: input.site_id.clone(),
        snapshot_id: input.snapshot_id.clone(),
        coordinate_reference_id: input.coordinate_reference_id.clone(),
        dataset_sha256: input.dataset_sha256.clone(),
        architecture_graph_sha256: input.architecture_graph_sha256.clone(),
        measurement_protocol_sha256: input.measurement_protocol_sha256.clone(),
        rights_decision_id: input.rights_decision_id.clone(),
        rights_receipt_sha256: input.rights_receipt_sha256.clone(),
        target_level,
        physical_stone_count: input.stones.len(),
        fixed_assigned_stone_count: fixed_assignments.len(),
        ambiguous_stone_count: ambiguous_assignments.len(),
        unassigned_stone_count,
        not_assessed_stone_count,
        scenario_count,
        maximum_scenarios: limits.maximum_scenarios,
        preflight_work_units,
        maximum_preflight_work_units: limits.maximum_work_units,
        groups,
        scenario_semantics:
            "unweighted_exhaustive_cartesian_product_of_per_stone_candidates_at_one_level_not_graph_constrained"
                .to_owned(),
        interval_interpretation:
            "outer_descriptive_envelopes_over_membership_scenarios_and_per_observation_measurement_bounds_not_confidence_or_credible_intervals"
                .to_owned(),
        claim_ceiling:
            "membership_sensitivity_of_descriptive_group_summaries_only_no_historical_or_social_inference"
                .to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heritage_morphology::{
        ARCHITECTURE_GROUP_LEVELS, GroupMembership, MeasurementInterval,
        StoneMorphologyObservation,
    };

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn assigned(level: ArchitectureGroupLevel) -> GroupMembership {
        GroupMembership {
            level,
            status: MembershipStatus::Assigned,
            group_id: Some(format!("group:{level:?}")),
            candidate_group_ids: Vec::new(),
        }
    }

    fn stone(id: &str, area: (f64, f64)) -> StoneMorphologyObservation {
        StoneMorphologyObservation {
            physical_identity_id: format!("physical:{id}"),
            observation_id: format!("observation:{id}"),
            geometry_sha256: digest('a'),
            source_capture_sha256: digest('b'),
            method_build_sha256: digest('c'),
            measurement_receipt_ref: format!("receipt:{id}"),
            measurement_receipt_sha256: digest('d'),
            plan_footprint_area_m2: MeasurementInterval {
                lower: area.0,
                upper: area.1,
            },
            mrr_area_ratio: Some(MeasurementInterval {
                lower: 0.8,
                upper: 0.9,
            }),
            elongation_ratio: Some(MeasurementInterval {
                lower: 1.1,
                upper: 1.3,
            }),
            memberships: ARCHITECTURE_GROUP_LEVELS
                .iter()
                .copied()
                .map(assigned)
                .collect(),
        }
    }

    fn input() -> HeritageMorphologyInput {
        HeritageMorphologyInput {
            schema_version: crate::heritage_morphology::HERITAGE_MORPHOLOGY_SCHEMA_VERSION
                .to_owned(),
            site_id: "synthetic-site-1".to_owned(),
            snapshot_id: "synthetic-snapshot-1".to_owned(),
            coordinate_reference_id: "EPSG:32735".to_owned(),
            dataset_sha256: digest('e'),
            architecture_graph_sha256: digest('f'),
            measurement_protocol_sha256: digest('1'),
            rights_decision_id: "rights:synthetic".to_owned(),
            rights_receipt_sha256: digest('2'),
            stones: vec![
                stone("a", (0.04, 0.05)),
                stone("b", (0.06, 0.07)),
                stone("c", (0.08, 0.09)),
            ],
        }
    }

    fn make_ambiguous(stone: &mut StoneMorphologyObservation, candidates: &[&str]) {
        let membership = stone
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        membership.status = MembershipStatus::Ambiguous;
        membership.group_id = None;
        membership.candidate_group_ids =
            candidates.iter().map(|candidate| (*candidate).to_owned()).collect();
    }

    #[test]
    fn exhaustively_enumerates_candidates_and_reports_group_envelopes() {
        let mut input = input();
        make_ambiguous(&mut input.stones[0], &["group:Wall", "alternate-wall"]);

        let report = evaluate_membership_sensitivity(
            &input,
            "sensitivity-1",
            ArchitectureGroupLevel::Wall,
            MembershipSensitivityLimits::default(),
        )
        .unwrap();

        assert_eq!(report.scenario_count, 2);
        assert_eq!(report.fixed_assigned_stone_count, 2);
        assert_eq!(report.ambiguous_stone_count, 1);
        assert_eq!(report.groups.len(), 2);

        let wall = report
            .groups
            .iter()
            .find(|group| group.group_id == "group:Wall")
            .unwrap();
        assert_eq!(wall.minimum_assigned_stone_count, 2);
        assert_eq!(wall.maximum_assigned_stone_count, 3);
        assert_eq!(wall.non_empty_scenario_count, 2);
        let mean = &wall.plan_footprint_area_m2.as_ref().unwrap().mean;
        assert!((mean.enclosing_interval.lower - 0.06).abs() < 1e-12);
        assert!((mean.enclosing_interval.upper - 0.08).abs() < 1e-12);
        assert!((mean.lower_endpoint_range.minimum - 0.06).abs() < 1e-12);
        assert!((mean.lower_endpoint_range.maximum - 0.07).abs() < 1e-12);
        assert!((mean.upper_endpoint_range.minimum - 0.07).abs() < 1e-12);
        assert!((mean.upper_endpoint_range.maximum - 0.08).abs() < 1e-12);

        let alternate = report
            .groups
            .iter()
            .find(|group| group.group_id == "alternate-wall")
            .unwrap();
        assert_eq!(alternate.minimum_assigned_stone_count, 0);
        assert_eq!(alternate.maximum_assigned_stone_count, 1);
        assert_eq!(alternate.non_empty_scenario_count, 1);
        assert_eq!(alternate.plan_footprint_area_m2.as_ref().unwrap().mean.enclosing_interval.lower, 0.04);
    }

    #[test]
    fn report_is_independent_of_input_and_candidate_order() {
        let mut first = input();
        make_ambiguous(&mut first.stones[0], &["z-wall", "a-wall"]);
        let first_report = evaluate_membership_sensitivity(
            &first,
            "sensitivity-1",
            ArchitectureGroupLevel::Wall,
            MembershipSensitivityLimits::default(),
        )
        .unwrap();

        let mut second = first.clone();
        second.stones.reverse();
        second.stones
            .iter_mut()
            .find(|stone| stone.physical_identity_id == "physical:a")
            .unwrap()
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap()
            .candidate_group_ids
            .reverse();

        let second_report = evaluate_membership_sensitivity(
            &second,
            "sensitivity-1",
            ArchitectureGroupLevel::Wall,
            MembershipSensitivityLimits::default(),
        )
        .unwrap();
        assert_eq!(first_report, second_report);
    }

    #[test]
    fn refuses_to_sample_or_truncate_when_scenario_cap_is_exceeded() {
        let mut input = input();
        make_ambiguous(&mut input.stones[0], &["group:Wall", "alternate-wall"]);
        let limits = MembershipSensitivityLimits {
            maximum_scenarios: 1,
            ..MembershipSensitivityLimits::default()
        };
        assert_eq!(
            evaluate_membership_sensitivity(
                &malformed,
                "sensitivity-1",
                ArchitectureGroupLevel::Wall,
                limits,
            ),
            Err(HeritageMembershipSensitivityError::ScenarioLimitExceeded {
                required: 2,
                maximum: 1,
            })
        );
    }

    #[test]
    fn operation_budget_is_enforced_before_enumeration() {
        let input = input();
        let limits = MembershipSensitivityLimits {
            maximum_work_units: 1,
            ..MembershipSensitivityLimits::default()
        };
        assert!(matches!(
            evaluate_membership_sensitivity(
                &input,
                "sensitivity-1",
                ArchitectureGroupLevel::Wall,
                limits,
            ),
            Err(HeritageMembershipSensitivityError::WorkLimitExceeded { .. })
        ));
    }

    #[test]
    fn unassigned_and_not_assessed_stones_never_become_candidates() {
        let mut input = input();
        make_ambiguous(&mut input.stones[0], &["group:Wall", "alternate-wall"]);
        let unassigned = input.stones[1]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        unassigned.status = MembershipStatus::Unassigned;
        unassigned.group_id = None;
        unassigned.candidate_group_ids.clear();
        let not_assessed = input.stones[2]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        not_assessed.status = MembershipStatus::NotAssessed;
        not_assessed.group_id = None;
        not_assessed.candidate_group_ids.clear();

        let report = evaluate_membership_sensitivity(
            &input,
            "sensitivity-1",
            ArchitectureGroupLevel::Wall,
            MembershipSensitivityLimits::default(),
        )
        .unwrap();
        assert_eq!(report.unassigned_stone_count, 1);
        assert_eq!(report.not_assessed_stone_count, 1);
        assert_eq!(report.fixed_assigned_stone_count, 0);
        assert_eq!(report.ambiguous_stone_count, 1);
        assert_eq!(report.scenario_count, 2);
    }

    #[test]
    fn no_ambiguity_is_one_exact_scenario() {
        let report = evaluate_membership_sensitivity(
            &input(),
            "sensitivity-1",
            ArchitectureGroupLevel::Wall,
            MembershipSensitivityLimits::default(),
        )
        .unwrap();
        assert_eq!(report.scenario_count, 1);
        let wall = report
            .groups
            .iter()
            .find(|group| group.group_id == "group:Wall")
            .unwrap();
        assert_eq!(wall.minimum_assigned_stone_count, 3);
        assert_eq!(wall.maximum_assigned_stone_count, 3);
        assert_eq!(wall.non_empty_scenario_count, 1);
    }

    #[test]
    fn malformed_input_and_empty_identifier_fail_closed() {
        let mut malformed = input();
        malformed.stones[0].memberships[0].candidate_group_ids.push("unexpected".into());
        assert!(matches!(
            evaluate_membership_sensitivity(
                &malformed,
                "sensitivity-1",
                ArchitectureGroupLevel::Wall,
                MembershipSensitivityLimits::default(),
            ),
            Err(HeritageMembershipSensitivityError::InvalidInput(_))
        ));

        assert_eq!(
            evaluate_membership_sensitivity(
                &input(),
                "  ",
                ArchitectureGroupLevel::Wall,
                MembershipSensitivityLimits::default(),
            ),
            Err(HeritageMembershipSensitivityError::EmptySensitivityId)
        );
    }
}
