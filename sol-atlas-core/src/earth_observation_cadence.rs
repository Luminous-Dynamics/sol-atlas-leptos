// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline qualification of actual usable satellite-observation cadence.
//!
//! Nominal orbit/revisit schedules, acquired scenes, quality-qualified
//! observations, data-delivery times and evidence-backed event-time bounds
//! are distinct quantities. This module scores a frozen timeline; it does not
//! fetch imagery, predict weather, command satellites, authenticate rights,
//! or certify a sensor's physical detection envelope.
//!
//! RightsBlocked is an upstream disposition emitted by the authoritative
//! rights/custody adapter. The module does not interpret grants or licences.
//! A no-change or change assessment must reference an independently qualified
//! detection-envelope receipt. Digests are references, not authentication.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CADENCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationModality {
    Optical,
    SyntheticApertureRadar,
    ThermalInfrared,
    Hyperspectral,
    Multispectral,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityState {
    /// An image/product was acquired, delivered and passed the declared EO QA gate.
    QualifiedUsable,
    CloudContaminated,
    ShadowContaminated,
    HazeOrSmokeContaminated,
    Saturated,
    IncompleteCoverage,
    InvalidProcessing,
    OutOfDomain,
    /// An acquisition exists but is not yet available to the analysis process.
    AwaitingDelivery,
    /// The source/custodian has not authorized the requested purpose.
    RightsBlocked,
    /// A known opportunity did not produce an acquisition.
    NoAcquisition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationAssessment {
    NoChangeWithinEnvelope,
    ChangeDetected,
    Unresolved,
}

/// Canonical UTC Unix-millisecond interval supplied by a qualified source-time
/// normalization step. Raw source timestamps and conversion receipts remain
/// referenced in the source/processing evidence; this type is not a time authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AcquisitionWindowWire")]
pub struct AcquisitionWindow {
    pub start_unix_ms: i64,
    pub end_unix_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcquisitionWindowWire {
    start_unix_ms: i64,
    end_unix_ms: i64,
}

impl TryFrom<AcquisitionWindowWire> for AcquisitionWindow {
    type Error = String;

    fn try_from(wire: AcquisitionWindowWire) -> Result<Self, Self::Error> {
        Self::new(wire.start_unix_ms, wire.end_unix_ms)
    }
}

impl AcquisitionWindow {
    pub fn new(start_unix_ms: i64, end_unix_ms: i64) -> Result<Self, String> {
        if start_unix_ms < 0 || end_unix_ms < start_unix_ms {
            return Err("acquisition window must be non-negative and ordered".into());
        }
        Ok(Self {
            start_unix_ms,
            end_unix_ms,
        })
    }

    pub fn midpoint_unix_ms(self) -> i64 {
        self.start_unix_ms + (self.end_unix_ms - self.start_unix_ms) / 2
    }
}

/// One expected acquisition opportunity or actual scene for one spatial support.
/// expected_at is optional: absence means no defensible acquisition schedule was
/// available; it must not be fabricated from nominal constellation marketing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CadenceOpportunityWire")]
pub struct CadenceOpportunity {
    pub opportunity_id: String,
    pub support_id: String,
    pub source_id: String,
    pub sensor_id: String,
    pub modality: ObservationModality,
    pub expected_at_unix_ms: Option<i64>,
    pub acquisition: Option<AcquisitionWindow>,
    pub asset_id: Option<String>,
    /// Source-native immutable acquisition/event identity, not a URL.
    pub source_event_id: Option<String>,
    pub state: OpportunityState,
    /// Time the qualified product became available to the analysis pipeline.
    pub delivered_at_unix_ms: Option<i64>,
    pub assessment: Option<ObservationAssessment>,
    /// Required for NoChangeWithinEnvelope: proves declared coverage over its
    /// support/window when authenticated by the responsible evidence owner.
    pub coverage_qualification_receipt_sha256: Option<String>,
    /// Required for both positive and negative promoted assessments: binds the
    /// target/support/sensor/platform/configuration detection envelope.
    pub detection_qualification_receipt_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CadenceOpportunityWire {
    opportunity_id: String,
    support_id: String,
    source_id: String,
    sensor_id: String,
    modality: ObservationModality,
    expected_at_unix_ms: Option<i64>,
    acquisition: Option<AcquisitionWindow>,
    asset_id: Option<String>,
    source_event_id: Option<String>,
    state: OpportunityState,
    delivered_at_unix_ms: Option<i64>,
    assessment: Option<ObservationAssessment>,
    coverage_qualification_receipt_sha256: Option<String>,
    detection_qualification_receipt_sha256: Option<String>,
}

impl TryFrom<CadenceOpportunityWire> for CadenceOpportunity {
    type Error = String;

    fn try_from(wire: CadenceOpportunityWire) -> Result<Self, Self::Error> {
        let opportunity = Self {
            opportunity_id: wire.opportunity_id,
            support_id: wire.support_id,
            source_id: wire.source_id,
            sensor_id: wire.sensor_id,
            modality: wire.modality,
            expected_at_unix_ms: wire.expected_at_unix_ms,
            acquisition: wire.acquisition,
            asset_id: wire.asset_id,
            source_event_id: wire.source_event_id,
            state: wire.state,
            delivered_at_unix_ms: wire.delivered_at_unix_ms,
            assessment: wire.assessment,
            coverage_qualification_receipt_sha256: wire.coverage_qualification_receipt_sha256,
            detection_qualification_receipt_sha256: wire.detection_qualification_receipt_sha256,
        };
        opportunity.validate()?;
        Ok(opportunity)
    }
}

impl CadenceOpportunity {
    fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.opportunity_id, "opportunity ID")?;
        validate_identifier(&self.support_id, "support ID")?;
        validate_identifier(&self.source_id, "source ID")?;
        validate_identifier(&self.sensor_id, "sensor ID")?;

        if self.expected_at_unix_ms.is_some_and(|time| time < 0) {
            return Err("expected acquisition time must be non-negative".into());
        }
        if self.delivered_at_unix_ms.is_some_and(|time| time < 0) {
            return Err("delivery time must be non-negative".into());
        }

        if self.state == OpportunityState::NoAcquisition {
            if self.expected_at_unix_ms.is_none() {
                return Err("no-acquisition opportunity requires an expected time".into());
            }
            if self.acquisition.is_some()
                || self.asset_id.is_some()
                || self.source_event_id.is_some()
                || self.delivered_at_unix_ms.is_some()
                || self.assessment.is_some()
                || self.coverage_qualification_receipt_sha256.is_some()
                || self.detection_qualification_receipt_sha256.is_some()
            {
                return Err("no-acquisition opportunity cannot carry acquired-scene fields".into());
            }
            return Ok(());
        }

        let acquisition = self
            .acquisition
            .ok_or_else(|| "acquired opportunity requires an acquisition interval".to_string())?;
        if acquisition.start_unix_ms < 0 || acquisition.end_unix_ms < acquisition.start_unix_ms {
            return Err("acquisition interval is invalid".into());
        }
        validate_optional_identifier(self.asset_id.as_deref(), "asset ID")?;
        validate_optional_identifier(self.source_event_id.as_deref(), "source event ID")?;
        if self.asset_id.is_none() || self.source_event_id.is_none() {
            return Err("acquired opportunity requires canonical asset and source-event identities".into());
        }

        if self.state == OpportunityState::QualifiedUsable && self.delivered_at_unix_ms.is_none() {
            return Err("qualified usable observation requires a delivery/availability time".into());
        }
        if self.state == OpportunityState::AwaitingDelivery && self.delivered_at_unix_ms.is_some() {
            return Err("awaiting-delivery state cannot carry a delivery time".into());
        }
        if self
            .delivered_at_unix_ms
            .is_some_and(|delivered| delivered < acquisition.end_unix_ms)
        {
            return Err("product cannot be available before acquisition completion".into());
        }

        if let Some(assessment) = self.assessment {
            if self.state != OpportunityState::QualifiedUsable {
                return Err("only a qualified usable observation may carry an assessment".into());
            }
            match assessment {
                ObservationAssessment::NoChangeWithinEnvelope => {
                    if self.coverage_qualification_receipt_sha256.is_none() {
                        return Err("no-change assessment requires a coverage qualification receipt".into());
                    }
                    if self.detection_qualification_receipt_sha256.is_none() {
                        return Err("no-change assessment requires a detection qualification receipt".into());
                    }
                }
                ObservationAssessment::ChangeDetected => {
                    if self.detection_qualification_receipt_sha256.is_none() {
                        return Err("change assessment requires a detection qualification receipt".into());
                    }
                }
                ObservationAssessment::Unresolved => {}
            }
        } else if self.coverage_qualification_receipt_sha256.is_some()
            || self.detection_qualification_receipt_sha256.is_some()
        {
            return Err("qualification receipts require an explicit observation assessment".into());
        }

        validate_optional_sha256(
            self.coverage_qualification_receipt_sha256.as_deref(),
            "coverage qualification receipt",
        )?;
        validate_optional_sha256(
            self.detection_qualification_receipt_sha256.as_deref(),
            "detection qualification receipt",
        )?;
        Ok(())
    }

    fn is_acquired(&self) -> bool {
        self.acquisition.is_some()
    }

    fn is_usable(&self) -> bool {
        self.state == OpportunityState::QualifiedUsable
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CadenceTimelineWire")]
pub struct CadenceTimeline {
    pub schema_version: u32,
    pub timeline_id: String,
    pub support_id: String,
    pub opportunities: Vec<CadenceOpportunity>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CadenceTimelineWire {
    schema_version: u32,
    timeline_id: String,
    support_id: String,
    opportunities: Vec<CadenceOpportunity>,
}

impl TryFrom<CadenceTimelineWire> for CadenceTimeline {
    type Error = String;

    fn try_from(wire: CadenceTimelineWire) -> Result<Self, Self::Error> {
        if wire.schema_version != CADENCE_SCHEMA_VERSION {
            return Err(format!(
                "unsupported cadence schema version {}; expected {}",
                wire.schema_version, CADENCE_SCHEMA_VERSION
            ));
        }
        Self::new(wire.timeline_id, wire.support_id, wire.opportunities)
    }
}

impl CadenceTimeline {
    pub fn new(
        timeline_id: impl Into<String>,
        support_id: impl Into<String>,
        mut opportunities: Vec<CadenceOpportunity>,
    ) -> Result<Self, String> {
        let timeline_id = timeline_id.into();
        let support_id = support_id.into();
        validate_identifier(&timeline_id, "timeline ID")?;
        validate_identifier(&support_id, "timeline support ID")?;
        if opportunities.is_empty() {
            return Err("cadence timeline must contain at least one opportunity".into());
        }

        let mut opportunity_ids = BTreeSet::new();
        let mut source_event_ids = BTreeSet::new();
        for opportunity in &opportunities {
            opportunity.validate()?;
            if opportunity.support_id != support_id {
                return Err("cadence timeline mixes spatial supports".into());
            }
            if !opportunity_ids.insert(opportunity.opportunity_id.as_str()) {
                return Err("duplicate opportunity ID".into());
            }
            if let Some(event_id) = opportunity.source_event_id.as_deref() {
                if !source_event_ids.insert(event_id) {
                    return Err("duplicate/replayed source event would inflate cadence".into());
                }
            }
        }

        // Canonical ordering makes serialization and downstream receipts deterministic.
        opportunities.sort_by(|a, b| {
            let a_time = a
                .acquisition
                .map(AcquisitionWindow::midpoint_unix_ms)
                .or(a.expected_at_unix_ms)
                .unwrap_or(i64::MAX);
            let b_time = b
                .acquisition
                .map(AcquisitionWindow::midpoint_unix_ms)
                .or(b.expected_at_unix_ms)
                .unwrap_or(i64::MAX);
            a_time.cmp(&b_time).then(a.opportunity_id.cmp(&b.opportunity_id))
        });

        Ok(Self {
            schema_version: CADENCE_SCHEMA_VERSION,
            timeline_id,
            support_id,
            opportunities,
        })
    }

    pub fn report(&self) -> CadenceReport {
        let expected_times = self
            .opportunities
            .iter()
            .filter_map(|item| item.expected_at_unix_ms)
            .collect::<Vec<_>>();
        let acquired_times = self
            .opportunities
            .iter()
            .filter_map(|item| item.acquisition.map(AcquisitionWindow::midpoint_unix_ms))
            .collect::<Vec<_>>();
        let usable_times = self
            .opportunities
            .iter()
            .filter(|item| item.is_usable())
            .filter_map(|item| item.acquisition.map(AcquisitionWindow::midpoint_unix_ms))
            .collect::<Vec<_>>();
        let delivery_latencies = self
            .opportunities
            .iter()
            .filter_map(|item| {
                Some(item.delivered_at_unix_ms? - item.acquisition?.end_unix_ms)
            })
            .collect::<Vec<_>>();

        let expected_count = expected_times.len();
        let acquired_count = self.opportunities.iter().filter(|item| item.is_acquired()).count();
        let usable_count = self.opportunities.iter().filter(|item| item.is_usable()).count();
        let usable_with_known_expected_count = self.opportunities
            .iter()
            .filter(|item| item.is_usable() && item.expected_at_unix_ms.is_some())
            .count();
        let no_acquisition_count = self
            .opportunities
            .iter()
            .filter(|item| item.state == OpportunityState::NoAcquisition)
            .count();
        let rights_blocked_count = self
            .opportunities
            .iter()
            .filter(|item| item.state == OpportunityState::RightsBlocked)
            .count();

        let usable_fraction_of_expected = (expected_count > 0)
            .then_some(usable_with_known_expected_count as f64 / expected_count as f64);
        let usable_fraction_of_acquired =
            (acquired_count > 0).then_some(usable_count as f64 / acquired_count as f64);

        let mut sensor_groups: BTreeMap<String, Vec<&CadenceOpportunity>> = BTreeMap::new();
        for opportunity in &self.opportunities {
            sensor_groups
                .entry(opportunity.sensor_id.clone())
                .or_default()
                .push(opportunity);
        }
        let per_sensor = sensor_groups
            .into_iter()
            .map(|(sensor_id, items)| {
                let acquired = items.iter().filter(|item| item.is_acquired()).count();
                let usable = items.iter().filter(|item| item.is_usable()).count();
                let times = items
                    .iter()
                    .filter(|item| item.is_usable())
                    .filter_map(|item| item.acquisition.map(AcquisitionWindow::midpoint_unix_ms))
                    .collect::<Vec<_>>();
                SensorCadenceSummary {
                    sensor_id,
                    acquired_count: acquired,
                    qualified_usable_count: usable,
                    usable_revisit_gaps: summarize_gaps(times),
                    rights_blocked_count: items
                        .iter()
                        .filter(|item| item.state == OpportunityState::RightsBlocked)
                        .count(),
                }
            })
            .collect();

        let first_actionable_claim_at_unix_ms = self
            .opportunities
            .iter()
            .filter(|item| {
                item.is_usable()
                    && matches!(
                        item.assessment,
                        Some(ObservationAssessment::NoChangeWithinEnvelope)
                            | Some(ObservationAssessment::ChangeDetected)
                    )
            })
            .filter_map(|item| item.delivered_at_unix_ms)
            .min();

        CadenceReport {
            timeline_id: self.timeline_id.clone(),
            support_id: self.support_id.clone(),
            opportunity_count: self.opportunities.len(),
            known_expected_opportunity_count: expected_count,
            acquired_count,
            qualified_usable_count: usable_count,
            no_acquisition_count,
            rights_blocked_count,
            usable_fraction_of_expected,
            usable_fraction_of_acquired,
            nominal_schedule_gaps: summarize_gaps(expected_times),
            actual_acquisition_gaps: summarize_gaps(acquired_times),
            usable_revisit_gaps: summarize_gaps(usable_times),
            delivery_latency: summarize_durations(delivery_latencies),
            first_actionable_claim_at_unix_ms,
            event_time_intervals: self.interval_censored_event_times(),
            per_sensor,
        }
    }

    /// Return one interval for each qualified positive assessment that has a
    /// preceding qualified no-change assessment on this same support.
    ///
    /// lower_bound_exclusive is the completion of the last qualifying no-change
    /// acquisition; upper_bound_inclusive is the start of the first positive
    /// acquisition. Unusable acquisitions inside the gap do not narrow it.
    pub fn interval_censored_event_times(&self) -> Vec<CensoredEventInterval> {
        let changes = self
            .opportunities
            .iter()
            .filter(|item| item.assessment == Some(ObservationAssessment::ChangeDetected))
            .filter_map(|item| {
                Some((
                    item,
                    item.acquisition?.start_unix_ms,
                ))
            })
            .collect::<Vec<_>>();

        let mut intervals = Vec::new();
        for (change, change_start) in changes {
            let prior_no_change = self
                .opportunities
                .iter()
                .filter(|item| item.assessment == Some(ObservationAssessment::NoChangeWithinEnvelope))
                .filter_map(|item| {
                    let window = item.acquisition?;
                    (window.end_unix_ms < change_start)
                        .then_some((item, window.end_unix_ms))
                })
                .max_by_key(|(_, end)| *end);

            if let Some((no_change, lower)) = prior_no_change {
                intervals.push(CensoredEventInterval {
                    last_qualified_no_change_opportunity_id: no_change.opportunity_id.clone(),
                    first_qualified_change_opportunity_id: change.opportunity_id.clone(),
                    lower_bound_exclusive_unix_ms: lower,
                    upper_bound_inclusive_unix_ms: change_start,
                    interval_width_ms: change_start - lower,
                });
            }
        }
        intervals.sort_by(|a, b| {
            a.upper_bound_inclusive_unix_ms
                .cmp(&b.upper_bound_inclusive_unix_ms)
                .then(a.first_qualified_change_opportunity_id.cmp(&b.first_qualified_change_opportunity_id))
        });
        intervals
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapSummary {
    pub gap_count: usize,
    pub p50_ms: Option<i64>,
    pub p90_ms: Option<i64>,
    pub maximum_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurationSummary {
    pub sample_count: usize,
    pub p50_ms: Option<i64>,
    pub p90_ms: Option<i64>,
    pub maximum_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorCadenceSummary {
    pub sensor_id: String,
    pub acquired_count: usize,
    pub qualified_usable_count: usize,
    pub rights_blocked_count: usize,
    pub usable_revisit_gaps: GapSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CensoredEventInterval {
    pub last_qualified_no_change_opportunity_id: String,
    pub first_qualified_change_opportunity_id: String,
    pub lower_bound_exclusive_unix_ms: i64,
    pub upper_bound_inclusive_unix_ms: i64,
    pub interval_width_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CadenceReport {
    pub timeline_id: String,
    pub support_id: String,
    pub opportunity_count: usize,
    pub known_expected_opportunity_count: usize,
    pub acquired_count: usize,
    pub qualified_usable_count: usize,
    pub no_acquisition_count: usize,
    pub rights_blocked_count: usize,
    pub usable_fraction_of_expected: Option<f64>,
    pub usable_fraction_of_acquired: Option<f64>,
    pub nominal_schedule_gaps: GapSummary,
    pub actual_acquisition_gaps: GapSummary,
    pub usable_revisit_gaps: GapSummary,
    /// Arrival delay distribution in milliseconds, not physical revisit.
    pub delivery_latency: DurationSummary,
    pub first_actionable_claim_at_unix_ms: Option<i64>,
    pub event_time_intervals: Vec<CensoredEventInterval>,
    pub per_sensor: Vec<SensorCadenceSummary>,
}

fn summarize_gaps(mut times: Vec<i64>) -> GapSummary {
    times.sort_unstable();
    let mut gaps = times
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect::<Vec<_>>();
    gaps.sort_unstable();
    let count = gaps.len();
    GapSummary {
        gap_count: count,
        p50_ms: percentile_nearest_rank(&gaps, 50),
        p90_ms: percentile_nearest_rank(&gaps, 90),
        maximum_ms: gaps.last().copied(),
    }
}

fn summarize_durations(mut durations: Vec<i64>) -> DurationSummary {
    durations.sort_unstable();
    DurationSummary {
        sample_count: durations.len(),
        p50_ms: percentile_nearest_rank(&durations, 50),
        p90_ms: percentile_nearest_rank(&durations, 90),
        maximum_ms: durations.last().copied(),
    }
}

fn percentile_nearest_rank(sorted_values: &[i64], percentile: usize) -> Option<i64> {
    if sorted_values.is_empty() || percentile == 0 || percentile > 100 {
        return None;
    }
    let rank = (percentile * sorted_values.len()).div_ceil(100).max(1);
    sorted_values.get(rank - 1).copied()
}

fn validate_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(format!("{label} must be a 1-128 character trimmed ASCII token"));
    }
    Ok(())
}

fn validate_optional_identifier(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        validate_identifier(value, label)?;
    }
    Ok(())
}

fn validate_optional_sha256(value: Option<&str>, label: &str) -> Result<(), String> {
    if let Some(value) = value {
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("{label} must be a 64-character SHA-256 hex digest"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400_000;

    fn item(
        id: &str,
        sensor: &str,
        modality: ObservationModality,
        expected: Option<i64>,
        window: Option<(i64, i64)>,
        state: OpportunityState,
        assessment: Option<ObservationAssessment>,
        delivered: Option<i64>,
    ) -> CadenceOpportunity {
        let (asset_id, source_event_id) = if state == OpportunityState::NoAcquisition {
            (None, None)
        } else {
            (Some(format!("asset:{id}")), Some(format!("event:{id}")))
        };
        let is_no_change = assessment == Some(ObservationAssessment::NoChangeWithinEnvelope);
        let is_claim = matches!(
            assessment,
            Some(ObservationAssessment::NoChangeWithinEnvelope)
                | Some(ObservationAssessment::ChangeDetected)
        );
        CadenceOpportunity {
            opportunity_id: id.to_string(),
            support_id: "support-a".into(),
            source_id: "source-a".into(),
            sensor_id: sensor.into(),
            modality,
            expected_at_unix_ms: expected,
            acquisition: window.map(|(start, end)| AcquisitionWindow {
                start_unix_ms: start,
                end_unix_ms: end,
            }),
            asset_id,
            source_event_id,
            state,
            delivered_at_unix_ms: delivered,
            assessment,
            coverage_qualification_receipt_sha256: is_no_change.then(|| "d".repeat(64)),
            detection_qualification_receipt_sha256: is_claim.then(|| "e".repeat(64)),
        }
    }

    #[test]
    fn usable_revisit_is_not_nominal_schedule_revisit() {
        let timeline = CadenceTimeline::new(
            "timeline-a",
            "support-a",
            vec![
                item("a", "optical-a", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::QualifiedUsable, None, Some(0)),
                item("b", "optical-a", ObservationModality::Optical, Some(5 * DAY), Some((5 * DAY, 5 * DAY)), OpportunityState::CloudContaminated, None, None),
                item("c", "optical-a", ObservationModality::Optical, Some(10 * DAY), Some((10 * DAY, 10 * DAY)), OpportunityState::HazeOrSmokeContaminated, None, None),
                item("d", "optical-a", ObservationModality::Optical, Some(15 * DAY), Some((15 * DAY, 15 * DAY)), OpportunityState::QualifiedUsable, None, Some(15 * DAY)),
                item("e", "optical-a", ObservationModality::Optical, Some(20 * DAY), Some((20 * DAY, 20 * DAY)), OpportunityState::QualifiedUsable, None, Some(20 * DAY)),
            ],
        ).unwrap();
        let report = timeline.report();
        assert_eq!(report.nominal_schedule_gaps.p50_ms, Some(5 * DAY));
        assert_eq!(report.usable_revisit_gaps.p50_ms, Some(5 * DAY));
        assert_eq!(report.usable_revisit_gaps.maximum_ms, Some(15 * DAY));
        assert_eq!(report.qualified_usable_count, 3);
        assert!((report.usable_fraction_of_expected.unwrap() - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn sar_can_supply_usable_observation_during_optical_cloud_gap() {
        let timeline = CadenceTimeline::new(
            "timeline-sar",
            "support-a",
            vec![
                item("optical", "sentinel-2", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::QualifiedUsable, None, Some(0)),
                item("cloudy", "sentinel-2", ObservationModality::Optical, Some(5 * DAY), Some((5 * DAY, 5 * DAY)), OpportunityState::CloudContaminated, None, None),
                item("sar", "sentinel-1", ObservationModality::SyntheticApertureRadar, Some(6 * DAY), Some((6 * DAY, 6 * DAY)), OpportunityState::QualifiedUsable, None, Some(6 * DAY)),
            ],
        ).unwrap();
        let report = timeline.report();
        assert_eq!(report.qualified_usable_count, 2);
        assert_eq!(report.per_sensor.len(), 2);
        assert_eq!(report.per_sensor.iter().find(|x| x.sensor_id == "sentinel-1").unwrap().qualified_usable_count, 1);
    }

    #[test]
    fn event_time_is_bounded_by_last_qualified_clean_and_first_positive() {
        let timeline = CadenceTimeline::new(
            "timeline-event",
            "support-a",
            vec![
                item("clean", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 10)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::NoChangeWithinEnvelope), Some(20)),
                item("cloud-gap", "sensor-a", ObservationModality::Optical, Some(DAY), Some((DAY, DAY + 10)), OpportunityState::CloudContaminated, None, None),
                item("positive", "sensor-a", ObservationModality::Optical, Some(2 * DAY), Some((2 * DAY, 2 * DAY + 10)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::ChangeDetected), Some(2 * DAY + 20)),
            ],
        ).unwrap();
        let intervals = timeline.interval_censored_event_times();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0].lower_bound_exclusive_unix_ms, 10);
        assert_eq!(intervals[0].upper_bound_inclusive_unix_ms, 2 * DAY);
        assert_eq!(intervals[0].interval_width_ms, 2 * DAY - 10);
    }

    #[test]
    fn no_change_requires_both_coverage_and_detection_qualification_receipts() {
        let mut invalid = item("clean", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 10)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::NoChangeWithinEnvelope), Some(20));
        invalid.detection_qualification_receipt_sha256 = None;
        assert!(CadenceTimeline::new("bad", "support-a", vec![invalid]).is_err());
    }

    #[test]
    fn duplicate_source_event_is_rejected_instead_of_inflating_revisit_count() {
        let first = item("scene-a", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::QualifiedUsable, None, Some(0));
        let mut duplicate = item("scene-b", "sensor-a", ObservationModality::Optical, Some(DAY), Some((DAY, DAY)), OpportunityState::QualifiedUsable, None, Some(DAY));
        duplicate.source_event_id = first.source_event_id.clone();
        assert!(CadenceTimeline::new("duplicate", "support-a", vec![first, duplicate]).is_err());
    }

    #[test]
    fn rights_blocked_and_no_acquisition_are_not_usable_observations() {
        let timeline = CadenceTimeline::new(
            "timeline-rights",
            "support-a",
            vec![
                item("blocked", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::RightsBlocked, None, None),
                item("missed", "sensor-a", ObservationModality::Optical, Some(DAY), None, OpportunityState::NoAcquisition, None, None),
            ],
        ).unwrap();
        let report = timeline.report();
        assert_eq!(report.qualified_usable_count, 0);
        assert_eq!(report.rights_blocked_count, 1);
        assert_eq!(report.no_acquisition_count, 1);
        assert_eq!(report.usable_fraction_of_expected, Some(0.0));
        assert!(report.usable_revisit_gaps.p50_ms.is_none());
    }

    #[test]
    fn delivery_latency_is_not_misreported_as_revisit() {
        let timeline = CadenceTimeline::new(
            "timeline-latency",
            "support-a",
            vec![
                item("late", "sensor-a", ObservationModality::Optical, Some(0), Some((0, DAY)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::ChangeDetected), Some(DAY + 3 * 60 * 60 * 1000)),
                item("later", "sensor-a", ObservationModality::Optical, Some(2 * DAY), Some((2 * DAY, 2 * DAY)), OpportunityState::QualifiedUsable, None, Some(2 * DAY)),
            ],
        ).unwrap();
        let report = timeline.report();
        assert_eq!(report.delivery_latency.sample_count, 2);
        assert_eq!(report.delivery_latency.p90_ms, Some(3 * 60 * 60 * 1000));
        assert_eq!(report.delivery_latency.maximum_ms, Some(3 * 60 * 60 * 1000));
        assert_eq!(report.usable_revisit_gaps.p50_ms, Some(DAY + DAY / 2));
        assert_eq!(report.first_actionable_claim_at_unix_ms, Some(DAY + 3 * 60 * 60 * 1000));
    }

    #[test]
    fn all_no_change_timeline_does_not_invent_a_hidden_transient_event() {
        let timeline = CadenceTimeline::new(
            "timeline-no-positive",
            "support-a",
            vec![
                item("clean-a", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::NoChangeWithinEnvelope), Some(0)),
                item("clean-b", "sensor-a", ObservationModality::Optical, Some(2 * DAY), Some((2 * DAY, 2 * DAY)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::NoChangeWithinEnvelope), Some(2 * DAY)),
            ],
        ).unwrap();
        assert!(timeline.interval_censored_event_times().is_empty());
    }

    #[test]
    fn committed_fixture_separates_optical_gap_from_sar_usability() {
        let fixture = include_str!("../tests/fixtures/earth_observation_cadence/optical_cloud_gap_sar_bridge.json");
        let timeline: CadenceTimeline = serde_json::from_str(fixture).unwrap();
        let report = timeline.report();
        assert_eq!(report.opportunity_count, 4);
        assert_eq!(report.acquired_count, 3);
        assert_eq!(report.qualified_usable_count, 2);
        assert_eq!(report.no_acquisition_count, 1);
        assert_eq!(report.usable_fraction_of_expected, Some(0.5));
        assert_eq!(report.per_sensor.len(), 2);
        assert_eq!(report.usable_revisit_gaps.gap_count, 1);
        assert_eq!(report.usable_revisit_gaps.maximum_ms, Some(518_400_000));
    }

    #[test]
    fn deserialized_timeline_revalidates_duplicate_and_receipt_invariants() {
        let valid = CadenceTimeline::new(
            "timeline-serialize",
            "support-a",
            vec![
                item("clean", "sensor-a", ObservationModality::Optical, Some(0), Some((0, 0)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::NoChangeWithinEnvelope), Some(0)),
                item("positive", "sensor-a", ObservationModality::Optical, Some(DAY), Some((DAY, DAY)), OpportunityState::QualifiedUsable, Some(ObservationAssessment::ChangeDetected), Some(DAY)),
            ],
        ).unwrap();
        let json = serde_json::to_string(&valid).unwrap();
        let decoded: CadenceTimeline = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, valid);
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value["opportunities"][0]["detection_qualification_receipt_sha256"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<CadenceTimeline>(value).is_err());
    }
}
