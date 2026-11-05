// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Uncertainty-bounded descriptive morphology summaries across architecture
//! hierarchy levels.
//!
//! This module consumes one selected, source-bound measurement per physical
//! stone. It preserves ambiguous/unassigned parentage and reports descriptive
//! distributions only. It does not fit population models, infer social identity,
//! infer chronology, or turn stone-level rows into independent site replicates.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_MORPHOLOGY_SCHEMA_VERSION: &str = "luminous.heritage-morphology.v0.1";
pub const MAX_MORPHOLOGY_STONES: usize = 2_000_000;
pub const ARCHITECTURE_GROUP_LEVELS: [ArchitectureGroupLevel; 8] = [
    ArchitectureGroupLevel::WallSegment,
    ArchitectureGroupLevel::Wall,
    ArchitectureGroupLevel::Building,
    ArchitectureGroupLevel::Household,
    ArchitectureGroupLevel::Compound,
    ArchitectureGroupLevel::Insula,
    ArchitectureGroupLevel::Neighbourhood,
    ArchitectureGroupLevel::Settlement,
];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementInterval {
    pub lower: f64,
    pub upper: f64,
}

impl MeasurementInterval {
    fn validate_positive(self, label: &str) -> Result<(), HeritageMorphologyError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower <= 0.0
            || self.lower > self.upper
        {
            return Err(HeritageMorphologyError::InvalidMeasurementInterval(
                label.into(),
            ));
        }
        Ok(())
    }

    fn validate_unit_interval(self, label: &str) -> Result<(), HeritageMorphologyError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower <= 0.0
            || self.upper > 1.0
            || self.lower > self.upper
        {
            return Err(HeritageMorphologyError::InvalidMeasurementInterval(
                label.into(),
            ));
        }
        Ok(())
    }

    fn validate_at_least_one(self, label: &str) -> Result<(), HeritageMorphologyError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower < 1.0
            || self.lower > self.upper
        {
            return Err(HeritageMorphologyError::InvalidMeasurementInterval(
                label.into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureGroupLevel {
    WallSegment,
    Wall,
    Building,
    Household,
    Compound,
    Insula,
    Neighbourhood,
    Settlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    Assigned,
    Ambiguous,
    Unassigned,
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupMembership {
    pub level: ArchitectureGroupLevel,
    pub status: MembershipStatus,
    pub group_id: Option<String>,
    pub candidate_group_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoneMorphologyObservation {
    /// Stable identity of the physical stone, not a frame/tile identifier.
    pub physical_identity_id: String,
    /// One selected measurement record per physical stone in this snapshot.
    pub observation_id: String,
    pub geometry_sha256: String,
    pub source_capture_sha256: String,
    pub method_build_sha256: String,
    pub measurement_receipt_ref: String,
    pub measurement_receipt_sha256: String,
    /// Calibrated planar footprint area in square metres; not 3D surface area.
    pub plan_footprint_area_m2: MeasurementInterval,
    /// Stone plan-footprint area / minimum-rotated-rectangle area.
    /// Values in (0, 1] near one indicate a more regular footprint.
    pub mrr_area_ratio: Option<MeasurementInterval>,
    /// Long-to-short axis ratio; values are >= 1.
    pub elongation_ratio: Option<MeasurementInterval>,
    /// Exactly one explicit membership state per architecture hierarchy level.
    pub memberships: Vec<GroupMembership>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageMorphologyInput {
    pub schema_version: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub dataset_sha256: String,
    pub architecture_graph_sha256: String,
    pub measurement_protocol_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_sha256: String,
    pub stones: Vec<StoneMorphologyObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalDistributionSummary {
    pub sample_count: usize,
    pub mean: MeasurementInterval,
    pub p10: MeasurementInterval,
    pub median: MeasurementInterval,
    pub p90: MeasurementInterval,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupMorphologySummary {
    pub level: ArchitectureGroupLevel,
    pub group_id: String,
    pub assigned_stone_count: usize,
    /// Distinct ambiguous stones for which this group is an eligible candidate.
    /// These stones are not included in assigned-only distribution summaries.
    pub ambiguous_candidate_stone_count: usize,
    pub plan_footprint_area_m2: Option<IntervalDistributionSummary>,
    pub mrr_area_ratio: Option<IntervalDistributionSummary>,
    pub elongation_ratio: Option<IntervalDistributionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelMembershipAudit {
    pub level: ArchitectureGroupLevel,
    pub assigned_stone_count: usize,
    pub ambiguous_stone_count: usize,
    pub unassigned_stone_count: usize,
    pub not_assessed_stone_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipAmbiguityRecord {
    pub physical_identity_id: String,
    pub level: ArchitectureGroupLevel,
    pub candidate_group_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageMorphologyReport {
    pub schema_version: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub dataset_sha256: String,
    pub architecture_graph_sha256: String,
    pub measurement_protocol_sha256: String,
    pub rights_decision_id: String,
    pub rights_receipt_sha256: String,
    pub physical_stone_count: usize,
    pub group_count: usize,
    pub groups: Vec<GroupMorphologySummary>,
    pub membership_audit: Vec<LevelMembershipAudit>,
    pub membership_ambiguities: Vec<MembershipAmbiguityRecord>,
    /// Per-observation measurement intervals are propagated to descriptive
    /// bounds only. These are not confidence or credible intervals.
    pub interval_interpretation: String,
    pub claim_ceiling: String,
    pub independent_site_replicates_claimed: usize,
    pub historical_interpretation_claimed: bool,
    pub social_identity_inference_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageMorphologyError {
    WrongSchemaVersion,
    EmptySiteId,
    EmptySnapshotId,
    EmptyCoordinateReferenceId,
    EmptyRightsDecisionId,
    InvalidDigest(String),
    RightsReceiptMissing,
    TooManyStones,
    EmptyPhysicalIdentityId,
    EmptyObservationId,
    DuplicatePhysicalIdentity(String),
    DuplicateObservationId(String),
    EmptyMeasurementReceiptRef(String),
    InvalidMeasurementInterval(String),
    MissingMembershipLevel {
        physical_identity_id: String,
        level: ArchitectureGroupLevel,
    },
    DuplicateMembershipLevel {
        physical_identity_id: String,
        level: ArchitectureGroupLevel,
    },
    InvalidMembership {
        physical_identity_id: String,
        level: ArchitectureGroupLevel,
    },
    EmptyGroupId {
        physical_identity_id: String,
        level: ArchitectureGroupLevel,
    },
    EmptyCandidateGroupId {
        physical_identity_id: String,
        level: ArchitectureGroupLevel,
    },
}

impl fmt::Display for HeritageMorphologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage morphology error: {self:?}")
    }
}

impl std::error::Error for HeritageMorphologyError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_digest(value: &str, label: &str) -> Result<(), HeritageMorphologyError> {
    if !valid_sha256(value) {
        return Err(HeritageMorphologyError::InvalidDigest(label.into()));
    }
    Ok(())
}

fn validate_membership(
    stone_id: &str,
    membership: &GroupMembership,
) -> Result<(), HeritageMorphologyError> {
    match membership.status {
        MembershipStatus::Assigned => {
            let group_id = membership.group_id.as_deref().ok_or(
                HeritageMorphologyError::InvalidMembership {
                    physical_identity_id: stone_id.into(),
                    level: membership.level,
                },
            )?;
            if !non_empty(group_id) {
                return Err(HeritageMorphologyError::EmptyGroupId {
                    physical_identity_id: stone_id.into(),
                    level: membership.level,
                });
            }
            if !membership.candidate_group_ids.is_empty() {
                return Err(HeritageMorphologyError::InvalidMembership {
                    physical_identity_id: stone_id.into(),
                    level: membership.level,
                });
            }
        }
        MembershipStatus::Ambiguous => {
            if membership.group_id.is_some() || membership.candidate_group_ids.len() < 2 {
                return Err(HeritageMorphologyError::InvalidMembership {
                    physical_identity_id: stone_id.into(),
                    level: membership.level,
                });
            }
            let mut candidates = BTreeSet::new();
            for candidate in &membership.candidate_group_ids {
                if !non_empty(candidate) {
                    return Err(HeritageMorphologyError::EmptyCandidateGroupId {
                        physical_identity_id: stone_id.into(),
                        level: membership.level,
                    });
                }
                if !candidates.insert(candidate.clone()) {
                    return Err(HeritageMorphologyError::InvalidMembership {
                        physical_identity_id: stone_id.into(),
                        level: membership.level,
                    });
                }
            }
        }
        MembershipStatus::Unassigned | MembershipStatus::NotAssessed => {
            if membership.group_id.is_some() || !membership.candidate_group_ids.is_empty() {
                return Err(HeritageMorphologyError::InvalidMembership {
                    physical_identity_id: stone_id.into(),
                    level: membership.level,
                });
            }
        }
    }
    Ok(())
}

fn validate_input(input: &HeritageMorphologyInput) -> Result<(), HeritageMorphologyError> {
    if input.schema_version != HERITAGE_MORPHOLOGY_SCHEMA_VERSION {
        return Err(HeritageMorphologyError::WrongSchemaVersion);
    }
    if !non_empty(&input.site_id) {
        return Err(HeritageMorphologyError::EmptySiteId);
    }
    if !non_empty(&input.snapshot_id) {
        return Err(HeritageMorphologyError::EmptySnapshotId);
    }
    if !non_empty(&input.coordinate_reference_id) {
        return Err(HeritageMorphologyError::EmptyCoordinateReferenceId);
    }
    if !non_empty(&input.rights_decision_id) {
        return Err(HeritageMorphologyError::EmptyRightsDecisionId);
    }
    for (digest, label) in [
        (&input.dataset_sha256, "dataset_sha256"),
        (
            &input.architecture_graph_sha256,
            "architecture_graph_sha256",
        ),
        (
            &input.measurement_protocol_sha256,
            "measurement_protocol_sha256",
        ),
        (&input.rights_receipt_sha256, "rights_receipt_sha256"),
    ] {
        validate_digest(digest, label)?;
    }
    if input.stones.len() > MAX_MORPHOLOGY_STONES {
        return Err(HeritageMorphologyError::TooManyStones);
    }

    let mut physical_ids = BTreeSet::new();
    let mut observation_ids = BTreeSet::new();
    for stone in &input.stones {
        if !non_empty(&stone.physical_identity_id) {
            return Err(HeritageMorphologyError::EmptyPhysicalIdentityId);
        }
        if !non_empty(&stone.observation_id) {
            return Err(HeritageMorphologyError::EmptyObservationId);
        }
        if !physical_ids.insert(stone.physical_identity_id.clone()) {
            return Err(HeritageMorphologyError::DuplicatePhysicalIdentity(
                stone.physical_identity_id.clone(),
            ));
        }
        if !observation_ids.insert(stone.observation_id.clone()) {
            return Err(HeritageMorphologyError::DuplicateObservationId(
                stone.observation_id.clone(),
            ));
        }
        for (digest, label) in [
            (&stone.geometry_sha256, "geometry_sha256"),
            (&stone.source_capture_sha256, "source_capture_sha256"),
            (&stone.method_build_sha256, "method_build_sha256"),
            (
                &stone.measurement_receipt_sha256,
                "measurement_receipt_sha256",
            ),
        ] {
            validate_digest(digest, label)?;
        }
        if !non_empty(&stone.measurement_receipt_ref) {
            return Err(HeritageMorphologyError::EmptyMeasurementReceiptRef(
                stone.physical_identity_id.clone(),
            ));
        }
        stone
            .plan_footprint_area_m2
            .validate_positive("plan_footprint_area_m2")?;
        if let Some(interval) = stone.mrr_area_ratio {
            interval.validate_unit_interval("mrr_area_ratio")?;
        }
        if let Some(interval) = stone.elongation_ratio {
            interval.validate_at_least_one("elongation_ratio")?;
        }

        let mut levels = BTreeSet::new();
        for membership in &stone.memberships {
            if !levels.insert(membership.level) {
                return Err(HeritageMorphologyError::DuplicateMembershipLevel {
                    physical_identity_id: stone.physical_identity_id.clone(),
                    level: membership.level,
                });
            }
            validate_membership(&stone.physical_identity_id, membership)?;
        }
        for level in ARCHITECTURE_GROUP_LEVELS {
            if !levels.contains(&level) {
                return Err(HeritageMorphologyError::MissingMembershipLevel {
                    physical_identity_id: stone.physical_identity_id.clone(),
                    level,
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
struct GroupAccumulator<'a> {
    assigned: Vec<&'a StoneMorphologyObservation>,
    ambiguous_candidate_ids: BTreeSet<String>,
}

fn nearest_rank_interval(
    intervals: &[MeasurementInterval],
    numerator: usize,
    denominator: usize,
) -> MeasurementInterval {
    let mut lower: Vec<f64> = intervals.iter().map(|interval| interval.lower).collect();
    let mut upper: Vec<f64> = intervals.iter().map(|interval| interval.upper).collect();
    lower.sort_by(f64::total_cmp);
    upper.sort_by(f64::total_cmp);
    let rank = (intervals.len() * numerator).div_ceil(denominator);
    let index = rank.saturating_sub(1).min(intervals.len() - 1);
    MeasurementInterval {
        lower: lower[index],
        upper: upper[index],
    }
}

fn summarize_intervals(intervals: &[MeasurementInterval]) -> Option<IntervalDistributionSummary> {
    if intervals.is_empty() {
        return None;
    }
    let count = intervals.len();
    let mean = MeasurementInterval {
        lower: intervals.iter().map(|value| value.lower).sum::<f64>() / count as f64,
        upper: intervals.iter().map(|value| value.upper).sum::<f64>() / count as f64,
    };
    Some(IntervalDistributionSummary {
        sample_count: count,
        mean,
        p10: nearest_rank_interval(intervals, 1, 10),
        median: nearest_rank_interval(intervals, 1, 2),
        p90: nearest_rank_interval(intervals, 9, 10),
    })
}

fn summarize_group(
    level: ArchitectureGroupLevel,
    group_id: String,
    accumulator: GroupAccumulator<'_>,
) -> GroupMorphologySummary {
    let area: Vec<MeasurementInterval> = accumulator
        .assigned
        .iter()
        .map(|stone| stone.plan_footprint_area_m2)
        .collect();
    let regularity: Vec<MeasurementInterval> = accumulator
        .assigned
        .iter()
        .filter_map(|stone| stone.mrr_area_ratio)
        .collect();
    let elongation: Vec<MeasurementInterval> = accumulator
        .assigned
        .iter()
        .filter_map(|stone| stone.elongation_ratio)
        .collect();
    GroupMorphologySummary {
        level,
        group_id,
        assigned_stone_count: accumulator.assigned.len(),
        ambiguous_candidate_stone_count: accumulator.ambiguous_candidate_ids.len(),
        plan_footprint_area_m2: summarize_intervals(&area),
        mrr_area_ratio: summarize_intervals(&regularity),
        elongation_ratio: summarize_intervals(&elongation),
    }
}

/// Produces deterministic descriptive morphology summaries. Only explicitly
/// assigned stones enter a group's statistics; ambiguous stones are retained
/// as candidate-membership records and counts rather than forced into a group.
pub fn aggregate_architecture_morphology(
    input: &HeritageMorphologyInput,
) -> Result<HeritageMorphologyReport, HeritageMorphologyError> {
    validate_input(input)?;

    let mut groups: BTreeMap<(ArchitectureGroupLevel, String), GroupAccumulator<'_>> =
        BTreeMap::new();
    let mut audits: BTreeMap<ArchitectureGroupLevel, [usize; 4]> = BTreeMap::new();
    let mut ambiguities = Vec::new();

    for stone in &input.stones {
        for membership in &stone.memberships {
            let audit = audits.entry(membership.level).or_default();
            match membership.status {
                MembershipStatus::Assigned => {
                    audit[0] += 1;
                    let group_id = membership.group_id.as_ref().expect("validated assignment");
                    groups
                        .entry((membership.level, group_id.clone()))
                        .or_default()
                        .assigned
                        .push(stone);
                }
                MembershipStatus::Ambiguous => {
                    audit[1] += 1;
                    ambiguities.push(MembershipAmbiguityRecord {
                        physical_identity_id: stone.physical_identity_id.clone(),
                        level: membership.level,
                        candidate_group_ids: {
                            let mut candidates = membership.candidate_group_ids.clone();
                            candidates.sort();
                            candidates
                        },
                    });
                    for candidate in &membership.candidate_group_ids {
                        groups
                            .entry((membership.level, candidate.clone()))
                            .or_default()
                            .ambiguous_candidate_ids
                            .insert(stone.physical_identity_id.clone());
                    }
                }
                MembershipStatus::Unassigned => audit[2] += 1,
                MembershipStatus::NotAssessed => audit[3] += 1,
            }
        }
    }

    let group_summaries: Vec<GroupMorphologySummary> = groups
        .into_iter()
        .map(|((level, group_id), mut accumulator)| {
            accumulator
                .assigned
                .sort_by(|left, right| left.physical_identity_id.cmp(&right.physical_identity_id));
            summarize_group(level, group_id, accumulator)
        })
        .collect();
    ambiguities.sort_by(|left, right| {
        left.physical_identity_id
            .cmp(&right.physical_identity_id)
            .then_with(|| left.level.cmp(&right.level))
    });
    let membership_audit: Vec<LevelMembershipAudit> = ARCHITECTURE_GROUP_LEVELS
        .iter()
        .map(|level| {
            let counts = audits.get(level).copied().unwrap_or_default();
            LevelMembershipAudit {
                level: *level,
                assigned_stone_count: counts[0],
                ambiguous_stone_count: counts[1],
                unassigned_stone_count: counts[2],
                not_assessed_stone_count: counts[3],
            }
        })
        .collect();

    Ok(HeritageMorphologyReport {
        schema_version: HERITAGE_MORPHOLOGY_SCHEMA_VERSION.into(),
        site_id: input.site_id.clone(),
        snapshot_id: input.snapshot_id.clone(),
        coordinate_reference_id: input.coordinate_reference_id.clone(),
        dataset_sha256: input.dataset_sha256.clone(),
        architecture_graph_sha256: input.architecture_graph_sha256.clone(),
        measurement_protocol_sha256: input.measurement_protocol_sha256.clone(),
        rights_decision_id: input.rights_decision_id.clone(),
        rights_receipt_sha256: input.rights_receipt_sha256.clone(),
        physical_stone_count: input.stones.len(),
        group_count: group_summaries.len(),
        groups: group_summaries,
        membership_audit,
        membership_ambiguities: ambiguities,
        interval_interpretation:
            "per_observation_rectangular_measurement_bounds_not_confidence_or_credible_intervals"
                .into(),
        claim_ceiling: "descriptive_architectural_morphology_only".into(),
        independent_site_replicates_claimed: 0,
        historical_interpretation_claimed: false,
        social_identity_inference_claimed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn membership(level: ArchitectureGroupLevel) -> GroupMembership {
        GroupMembership {
            level,
            status: MembershipStatus::Assigned,
            group_id: Some(format!("group:{level:?}")),
            candidate_group_ids: Vec::new(),
        }
    }

    fn stone(id: &str, area: MeasurementInterval) -> StoneMorphologyObservation {
        StoneMorphologyObservation {
            physical_identity_id: format!("physical:{id}"),
            observation_id: format!("observation:{id}"),
            geometry_sha256: digest('a'),
            source_capture_sha256: digest('b'),
            method_build_sha256: digest('c'),
            measurement_receipt_ref: format!("receipt:{id}"),
            measurement_receipt_sha256: digest('d'),
            plan_footprint_area_m2: area,
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
                .map(membership)
                .collect(),
        }
    }

    fn input() -> HeritageMorphologyInput {
        HeritageMorphologyInput {
            schema_version: HERITAGE_MORPHOLOGY_SCHEMA_VERSION.into(),
            site_id: "synthetic-site-1".into(),
            snapshot_id: "synthetic-snapshot-1".into(),
            coordinate_reference_id: "EPSG:32735".into(),
            dataset_sha256: digest('e'),
            architecture_graph_sha256: digest('f'),
            measurement_protocol_sha256: digest('1'),
            rights_decision_id: "rights:synthetic".into(),
            rights_receipt_sha256: digest('2'),
            stones: vec![
                stone(
                    "a",
                    MeasurementInterval {
                        lower: 0.04,
                        upper: 0.05,
                    },
                ),
                stone(
                    "b",
                    MeasurementInterval {
                        lower: 0.06,
                        upper: 0.07,
                    },
                ),
                stone(
                    "c",
                    MeasurementInterval {
                        lower: 0.08,
                        upper: 0.09,
                    },
                ),
            ],
        }
    }

    #[test]
    fn aggregates_area_shape_and_elongation_with_interval_bounds() {
        let report = aggregate_architecture_morphology(&input()).unwrap();
        let wall = report
            .groups
            .iter()
            .find(|group| {
                group.level == ArchitectureGroupLevel::Wall && group.group_id == "group:Wall"
            })
            .unwrap();
        assert_eq!(wall.assigned_stone_count, 3);
        let area = wall.plan_footprint_area_m2.unwrap();
        assert_eq!(area.sample_count, 3);
        assert!((area.mean.lower - 0.06).abs() < 1e-12);
        assert!((area.mean.upper - 0.07).abs() < 1e-12);
        assert_eq!(area.median.lower, 0.06);
        assert_eq!(area.median.upper, 0.07);
        assert_eq!(wall.mrr_area_ratio.unwrap().sample_count, 3);
        assert_eq!(wall.elongation_ratio.unwrap().sample_count, 3);
    }

    #[test]
    fn ambiguous_membership_is_preserved_and_not_added_to_assigned_statistics() {
        let mut input = input();
        let membership = input.stones[0]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        membership.status = MembershipStatus::Ambiguous;
        membership.group_id = None;
        membership.candidate_group_ids = vec!["wall-a".into(), "wall-b".into()];
        let report = aggregate_architecture_morphology(&input).unwrap();
        assert_eq!(report.membership_ambiguities.len(), 1);
        let audit = report
            .membership_audit
            .iter()
            .find(|audit| audit.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        assert_eq!(audit.assigned_stone_count, 2);
        assert_eq!(audit.ambiguous_stone_count, 1);
        let wall_a = report
            .groups
            .iter()
            .find(|group| group.level == ArchitectureGroupLevel::Wall && group.group_id == "wall-a")
            .unwrap();
        assert_eq!(wall_a.assigned_stone_count, 0);
        assert_eq!(wall_a.ambiguous_candidate_stone_count, 1);
        assert_eq!(wall_a.plan_footprint_area_m2, None);
    }

    #[test]
    fn duplicate_physical_identity_is_not_counted_twice() {
        let mut input = input();
        input.stones[1].physical_identity_id = input.stones[0].physical_identity_id.clone();
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::DuplicatePhysicalIdentity(_))
        ));
    }

    #[test]
    fn duplicate_observation_identity_is_rejected() {
        let mut input = input();
        input.stones[1].observation_id = input.stones[0].observation_id.clone();
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::DuplicateObservationId(_))
        ));
    }

    #[test]
    fn unassigned_and_not_assessed_membership_are_separate() {
        let mut input = input();
        for (stone, status) in [
            (&mut input.stones[0], MembershipStatus::Unassigned),
            (&mut input.stones[1], MembershipStatus::NotAssessed),
        ] {
            let member = stone
                .memberships
                .iter_mut()
                .find(|member| member.level == ArchitectureGroupLevel::Household)
                .unwrap();
            member.status = status;
            member.group_id = None;
        }
        let report = aggregate_architecture_morphology(&input).unwrap();
        let audit = report
            .membership_audit
            .iter()
            .find(|audit| audit.level == ArchitectureGroupLevel::Household)
            .unwrap();
        assert_eq!(audit.unassigned_stone_count, 1);
        assert_eq!(audit.not_assessed_stone_count, 1);
    }

    #[test]
    fn missing_or_duplicate_hierarchy_level_is_rejected() {
        let mut missing_input = input();
        missing_input.stones[0]
            .memberships
            .retain(|membership| membership.level != ArchitectureGroupLevel::Insula);
        assert!(matches!(
            aggregate_architecture_morphology(&missing_input),
            Err(HeritageMorphologyError::MissingMembershipLevel { .. })
        ));
        let mut duplicate_input = input();
        duplicate_input.stones[0]
            .memberships
            .push(membership(ArchitectureGroupLevel::Wall));
        assert!(matches!(
            aggregate_architecture_morphology(&duplicate_input),
            Err(HeritageMorphologyError::DuplicateMembershipLevel { .. })
        ));
    }

    #[test]
    fn invalid_membership_status_combinations_fail_closed() {
        let mut input = input();
        let member = input.stones[0]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        member.status = MembershipStatus::Ambiguous;
        member.group_id = Some("forced-wall".into());
        member.candidate_group_ids = vec!["wall-a".into(), "wall-b".into()];
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::InvalidMembership { .. })
        ));
    }

    #[test]
    fn invalid_morphology_intervals_fail_closed() {
        let mut input = input();
        input.stones[0].plan_footprint_area_m2 = MeasurementInterval {
            lower: 0.1,
            upper: 0.05,
        };
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::InvalidMeasurementInterval(_))
        ));
        input = input();
        input.stones[0].mrr_area_ratio = Some(MeasurementInterval {
            lower: 0.9,
            upper: 1.1,
        });
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::InvalidMeasurementInterval(_))
        ));
    }

    #[test]
    fn input_order_does_not_change_group_reports() {
        let mut input = input();
        let first = aggregate_architecture_morphology(&input).unwrap();
        input.stones.reverse();
        let second = aggregate_architecture_morphology(&input).unwrap();
        assert_eq!(first.groups, second.groups);
        assert_eq!(first.membership_audit, second.membership_audit);
    }

    #[test]
    fn ambiguous_membership_order_is_canonicalized() {
        let mut input = input();
        let member = input.stones[0]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        member.status = MembershipStatus::Ambiguous;
        member.group_id = None;
        member.candidate_group_ids = vec!["wall-z".into(), "wall-a".into()];
        let first = aggregate_architecture_morphology(&input).unwrap();
        input.stones[0]
            .memberships
            .iter_mut()
            .find(|membership| membership.level == ArchitectureGroupLevel::Wall)
            .unwrap()
            .candidate_group_ids
            .reverse();
        let second = aggregate_architecture_morphology(&input).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn group_summary_is_descriptive_not_a_site_replication_claim() {
        let report = aggregate_architecture_morphology(&input()).unwrap();
        assert_eq!(report.independent_site_replicates_claimed, 0);
        assert!(!report.historical_interpretation_claimed);
        assert!(!report.social_identity_inference_claimed);
        assert_eq!(
            report.claim_ceiling,
            "descriptive_architectural_morphology_only"
        );
    }

    #[test]
    fn absent_shape_descriptor_stays_absent_not_zero() {
        let mut input = input();
        input.stones[0].mrr_area_ratio = None;
        input.stones[1].mrr_area_ratio = None;
        input.stones[2].mrr_area_ratio = None;
        let report = aggregate_architecture_morphology(&input).unwrap();
        let wall = report
            .groups
            .iter()
            .find(|group| group.level == ArchitectureGroupLevel::Wall)
            .unwrap();
        assert_eq!(wall.mrr_area_ratio, None);
    }

    #[test]
    fn malformed_provenance_digest_fails_closed() {
        let mut input = input();
        input.architecture_graph_sha256 = "not-a-digest".into();
        assert!(matches!(
            aggregate_architecture_morphology(&input),
            Err(HeritageMorphologyError::InvalidDigest(_))
        ));
    }

    #[test]
    fn synthetic_morphology_fixture_round_trips_through_serde() {
        let fixture =
            include_str!("../../schemas/examples/heritage-morphology-synthetic-v0.1.json");
        let input: HeritageMorphologyInput = serde_json::from_str(fixture).unwrap();
        let report = aggregate_architecture_morphology(&input).unwrap();
        assert_eq!(report.physical_stone_count, 1);
        assert_eq!(report.groups.len(), 8);
    }
}
