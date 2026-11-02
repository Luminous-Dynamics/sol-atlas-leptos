// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Metrics for archaeological heritage-threat monitoring.
//!
//! This module evaluates site-level temporal alert series; it does not ingest
//! imagery, detect looting, or decide whether an incident occurred. Labels are
//! explicit: known complete event windows, verified no-event windows, or
//! unknown. Missing observation months are never imputed as negative evidence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const THREAT_SERIES_SCHEMA_VERSION: &str = "luminous.heritage-threat-series.v0.1";
pub const THREAT_REPORT_SCHEMA_VERSION: &str = "luminous.heritage-threat-report.v0.1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatMonthObservation {
    /// Integer month ordinal: year * 12 + (month - 1), where month is 1..=12.
    pub month_ordinal: i64,
    /// None means no valid observation for this month, not zero threat.
    pub alert_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThreatGroundTruth {
    /// The supplied event list is complete for the inclusive monitoring window.
    KnownEvents {
        monitoring_start_month: i64,
        monitoring_end_month: i64,
        event_months: Vec<i64>,
    },
    /// A custodian/researcher verified that no target event occurred in this
    /// inclusive interval. Only this state creates negative evaluation months.
    ConfirmedNoEventForWindow {
        monitoring_start_month: i64,
        monitoring_end_month: i64,
    },
    /// No event-level ground truth is available. This series is excluded from
    /// event and false-alert metrics.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatMonitoringSeries {
    pub schema_version: String,
    pub series_id: String,
    pub site_id: String,
    pub ground_truth: ThreatGroundTruth,
    /// Strictly increasing unique month ordinals. Months absent from this list
    /// and observations with alert_score=None are both unobserved.
    pub observations: Vec<ThreatMonthObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThreatEvaluationConfig {
    /// A score at or above this value generates an alert month.
    pub alert_threshold: f64,
    /// Predicted alert episodes within this many months of an event are matched.
    pub temporal_tolerance_months: u32,
    /// Merge alert months when the number of intervening unalerted months is
    /// less than or equal to this value.
    pub alert_merge_gap_months: u32,
    /// An alert 1..=N months before an event counts as an early warning.
    pub early_warning_window_months: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatMonitoringReport {
    pub schema_version: String,
    pub site_series_count: usize,
    pub known_event_site_count: usize,
    pub confirmed_no_event_site_count: usize,
    pub unknown_site_count: usize,
    pub known_event_count: usize,
    pub matched_event_count: usize,
    pub exact_month_matched_event_count: usize,
    pub event_recall_within_tolerance: Option<f64>,
    pub exact_month_event_recall: Option<f64>,
    pub median_absolute_localisation_error_months: Option<f64>,
    pub early_warning_event_count: usize,
    pub early_warning_event_rate: Option<f64>,
    pub median_early_warning_lead_months: Option<f64>,
    pub alert_episode_count: usize,
    pub false_alert_count: usize,
    pub expected_labelled_site_months: u64,
    pub observed_labelled_site_months: u64,
    pub observation_coverage: Option<f64>,
    pub false_alerts_per_100_observed_site_months: Option<f64>,
    pub alert_threshold: f64,
    pub temporal_tolerance_months: u32,
    pub alert_merge_gap_months: u32,
    pub early_warning_window_months: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatEvaluationError {
    InvalidConfig,
    EmptySeriesId,
    EmptySiteId,
    DuplicateSiteId(String),
    UnsupportedSchemaVersion { series_id: String },
    InvalidMonitoringWindow { series_id: String },
    KnownEventsListEmpty { series_id: String },
    EventMonthOutsideWindow { series_id: String, month: i64 },
    EventMonthsNotStrictlyIncreasing { series_id: String },
    ObservationsNotStrictlyIncreasing { series_id: String },
    InvalidAlertScore { series_id: String, month: i64 },
}

impl fmt::Display for ThreatEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage threat evaluation input error: {self:?}")
    }
}

impl std::error::Error for ThreatEvaluationError {}

/// Convert a Gregorian year/month into the series' integer month ordinal.
pub fn month_ordinal(year: i32, month: u8) -> Option<i64> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    i64::from(year)
        .checked_mul(12)?
        .checked_add(i64::from(month) - 1)
}

/// Convert a month ordinal back to a Gregorian (year, month) pair.
pub fn year_month_from_ordinal(ordinal: i64) -> Option<(i32, u8)> {
    let year = ordinal.div_euclid(12);
    let month = ordinal.rem_euclid(12) + 1;
    if !(1..=9999).contains(&year) {
        return None;
    }
    Some((i32::try_from(year).ok()?, u8::try_from(month).ok()?))
}

fn validate_series(series: &ThreatMonitoringSeries) -> Result<(), ThreatEvaluationError> {
    if series.schema_version != THREAT_SERIES_SCHEMA_VERSION {
        return Err(ThreatEvaluationError::UnsupportedSchemaVersion {
            series_id: series.series_id.clone(),
        });
    }
    if series.series_id.trim().is_empty() {
        return Err(ThreatEvaluationError::EmptySeriesId);
    }
    if series.site_id.trim().is_empty() {
        return Err(ThreatEvaluationError::EmptySiteId);
    }

    let validate_window = |start: i64, end: i64| {
        start <= end
            && year_month_from_ordinal(start).is_some()
            && year_month_from_ordinal(end).is_some()
    };

    match &series.ground_truth {
        ThreatGroundTruth::KnownEvents {
            monitoring_start_month,
            monitoring_end_month,
            event_months,
        } => {
            if !validate_window(*monitoring_start_month, *monitoring_end_month) {
                return Err(ThreatEvaluationError::InvalidMonitoringWindow {
                    series_id: series.series_id.clone(),
                });
            }
            if event_months.is_empty() {
                return Err(ThreatEvaluationError::KnownEventsListEmpty {
                    series_id: series.series_id.clone(),
                });
            }
            let mut previous = None;
            for month in event_months {
                if month < monitoring_start_month || month > monitoring_end_month {
                    return Err(ThreatEvaluationError::EventMonthOutsideWindow {
                        series_id: series.series_id.clone(),
                        month: *month,
                    });
                }
                if previous.is_some_and(|last| *month <= last) {
                    return Err(ThreatEvaluationError::EventMonthsNotStrictlyIncreasing {
                        series_id: series.series_id.clone(),
                    });
                }
                previous = Some(*month);
            }
        }
        ThreatGroundTruth::ConfirmedNoEventForWindow {
            monitoring_start_month,
            monitoring_end_month,
        } => {
            if !validate_window(*monitoring_start_month, *monitoring_end_month) {
                return Err(ThreatEvaluationError::InvalidMonitoringWindow {
                    series_id: series.series_id.clone(),
                });
            }
        }
        ThreatGroundTruth::Unknown => {}
    }

    let mut previous = None;
    for observation in &series.observations {
        if year_month_from_ordinal(observation.month_ordinal).is_none()
            || previous.is_some_and(|last| observation.month_ordinal <= last)
        {
            return Err(ThreatEvaluationError::ObservationsNotStrictlyIncreasing {
                series_id: series.series_id.clone(),
            });
        }
        if let Some(score) = observation.alert_score {
            if !score.is_finite() || !(0.0..=1.0).contains(&score) {
                return Err(ThreatEvaluationError::InvalidAlertScore {
                    series_id: series.series_id.clone(),
                    month: observation.month_ordinal,
                });
            }
        }
        previous = Some(observation.month_ordinal);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct AlertEpisode {
    month: i64,
    score: f64,
}

fn alert_episodes(
    observations: &[ThreatMonthObservation],
    window_start: i64,
    window_end: i64,
    config: ThreatEvaluationConfig,
) -> Vec<AlertEpisode> {
    let mut episodes: Vec<AlertEpisode> = Vec::new();
    let mut last_alert_month = None;
    let max_distance = i64::from(config.alert_merge_gap_months) + 1;

    for observation in observations {
        if observation.month_ordinal < window_start || observation.month_ordinal > window_end {
            continue;
        }
        let Some(score) = observation.alert_score else {
            continue;
        };
        if score < config.alert_threshold {
            continue;
        }

        if let Some(last_month) = last_alert_month {
            if observation.month_ordinal - last_month <= max_distance {
                if let Some(current) = episodes.last_mut() {
                    // Keep the strongest score; for ties retain the earlier month.
                    if score > current.score {
                        current.month = observation.month_ordinal;
                        current.score = score;
                    }
                }
                last_alert_month = Some(observation.month_ordinal);
                continue;
            }
        }

        episodes.push(AlertEpisode {
            month: observation.month_ordinal,
            score,
        });
        last_alert_month = Some(observation.month_ordinal);
    }

    episodes.sort_by_key(|episode| episode.month);
    episodes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchChoice {
    Match,
    SkipEvent,
    SkipAlert,
}

#[derive(Debug, Clone, Copy, Default)]
struct MatchCell {
    count: usize,
    total_error: u64,
    choice: Option<MatchChoice>,
}

fn is_better_match(candidate: MatchCell, current: MatchCell) -> bool {
    if candidate.count != current.count {
        return candidate.count > current.count;
    }
    if candidate.total_error != current.total_error {
        return candidate.total_error < current.total_error;
    }
    let preference = |choice: Option<MatchChoice>| match choice {
        Some(MatchChoice::Match) => 0,
        Some(MatchChoice::SkipEvent) => 1,
        Some(MatchChoice::SkipAlert) => 2,
        None => 3,
    };
    preference(candidate.choice) < preference(current.choice)
}

/// Maximum-cardinality, minimum-total-error one-to-one matching for two
/// chronological sequences. The predicate returns a match cost when a pair is
/// allowed. Dynamic programming avoids double-counting one alert for repeated
/// events and produces deterministic tie-breaking.
fn optimal_event_matches(
    events: &[i64],
    alerts: &[AlertEpisode],
    pair_cost: impl Fn(i64, i64) -> Option<u64>,
) -> Vec<(usize, usize, u64)> {
    let n = events.len();
    let m = alerts.len();
    let mut cells = vec![vec![MatchCell::default(); m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            let mut best = cells[i - 1][j];
            best.choice = Some(MatchChoice::SkipEvent);

            let mut skip_alert = cells[i][j - 1];
            skip_alert.choice = Some(MatchChoice::SkipAlert);
            if is_better_match(skip_alert, best) {
                best = skip_alert;
            }

            if let Some(cost) = pair_cost(events[i - 1], alerts[j - 1].month) {
                let previous = cells[i - 1][j - 1];
                let matched = MatchCell {
                    count: previous.count + 1,
                    total_error: previous.total_error.saturating_add(cost),
                    choice: Some(MatchChoice::Match),
                };
                if is_better_match(matched, best) {
                    best = matched;
                }
            }
            cells[i][j] = best;
        }
    }

    let mut matches = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        match cells[i][j].choice {
            Some(MatchChoice::Match) => {
                let cost = pair_cost(events[i - 1], alerts[j - 1].month)
                    .expect("matched DP cell must satisfy pair predicate");
                matches.push((i - 1, j - 1, cost));
                i -= 1;
                j -= 1;
            }
            Some(MatchChoice::SkipEvent) => i -= 1,
            Some(MatchChoice::SkipAlert) => j -= 1,
            None => break,
        }
    }
    matches.reverse();
    matches
}

fn median_u64(values: &mut [u64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[middle] as f64)
    } else {
        Some((values[middle - 1] as f64 + values[middle] as f64) / 2.0)
    }
}

/// Evaluate temporal alert streams against explicit event/no-event labels.
///
/// KnownEvents requires a complete event list for its entire monitoring window.
/// ConfirmedNoEventForWindow is the only state that contributes verified
/// negative months. Unknown contributes neither positives nor negatives.
/// Missing observations are not assigned a score of zero.
///
/// Event matching is one-to-one and permits repeated events at one site.
/// False alerts are unmatched alert episodes within explicitly labelled
/// windows. Recall and false-alert burden are site/event-level, not pixel-level.
pub fn evaluate_threat_monitoring(
    series: &[ThreatMonitoringSeries],
    config: ThreatEvaluationConfig,
) -> Result<ThreatMonitoringReport, ThreatEvaluationError> {
    if !config.alert_threshold.is_finite() || !(0.0..=1.0).contains(&config.alert_threshold) {
        return Err(ThreatEvaluationError::InvalidConfig);
    }

    let mut site_ids = BTreeSet::new();
    let mut known_event_site_count = 0;
    let mut confirmed_no_event_site_count = 0;
    let mut unknown_site_count = 0;
    let mut known_event_count = 0;
    let mut matched_event_count = 0;
    let mut exact_month_matched_event_count = 0;
    let mut early_warning_event_count = 0;
    let mut alert_episode_count = 0;
    let mut false_alert_count = 0;
    let mut expected_labelled_site_months = 0_u64;
    let mut observed_labelled_site_months = 0_u64;
    let mut absolute_errors = Vec::new();
    let mut early_warning_leads = Vec::new();

    for site in series {
        validate_series(site)?;
        if !site_ids.insert(site.site_id.as_str()) {
            return Err(ThreatEvaluationError::DuplicateSiteId(site.site_id.clone()));
        }

        match &site.ground_truth {
            ThreatGroundTruth::Unknown => {
                unknown_site_count += 1;
            }
            ThreatGroundTruth::ConfirmedNoEventForWindow {
                monitoring_start_month,
                monitoring_end_month,
            } => {
                confirmed_no_event_site_count += 1;
                let total_months = u64::try_from(monitoring_end_month - monitoring_start_month + 1)
                    .expect("validated positive month span must fit u64");
                let observed_months = site
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.month_ordinal >= *monitoring_start_month
                            && observation.month_ordinal <= *monitoring_end_month
                            && observation.alert_score.is_some()
                    })
                    .count() as u64;
                expected_labelled_site_months =
                    expected_labelled_site_months.saturating_add(total_months);
                observed_labelled_site_months =
                    observed_labelled_site_months.saturating_add(observed_months);

                let alerts = alert_episodes(
                    &site.observations,
                    *monitoring_start_month,
                    *monitoring_end_month,
                    config,
                );
                alert_episode_count += alerts.len();
                false_alert_count += alerts.len();
            }
            ThreatGroundTruth::KnownEvents {
                monitoring_start_month,
                monitoring_end_month,
                event_months,
            } => {
                known_event_site_count += 1;
                known_event_count += event_months.len();
                let total_months = u64::try_from(monitoring_end_month - monitoring_start_month + 1)
                    .expect("validated positive month span must fit u64");
                let observed_months = site
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.month_ordinal >= *monitoring_start_month
                            && observation.month_ordinal <= *monitoring_end_month
                            && observation.alert_score.is_some()
                    })
                    .count() as u64;
                expected_labelled_site_months =
                    expected_labelled_site_months.saturating_add(total_months);
                observed_labelled_site_months =
                    observed_labelled_site_months.saturating_add(observed_months);

                let alerts = alert_episodes(
                    &site.observations,
                    *monitoring_start_month,
                    *monitoring_end_month,
                    config,
                );
                alert_episode_count += alerts.len();

                let tolerance = u64::from(config.temporal_tolerance_months);
                let matches = optimal_event_matches(event_months, &alerts, |event, alert| {
                    let distance = event.abs_diff(alert);
                    (distance <= tolerance).then_some(distance)
                });
                matched_event_count += matches.len();
                exact_month_matched_event_count +=
                    matches.iter().filter(|(_, _, error)| *error == 0).count();
                absolute_errors.extend(matches.iter().map(|(_, _, error)| *error));

                let early_matches = if config.early_warning_window_months > 0 {
                    let early_window = u64::from(config.early_warning_window_months);
                    optimal_event_matches(event_months, &alerts, |event, alert| {
                        let lead = event.checked_sub(alert)?;
                        (lead > 0 && lead as u64 <= early_window).then_some(lead as u64)
                    })
                } else {
                    Vec::new()
                };
                early_warning_event_count += early_matches.len();
                early_warning_leads.extend(early_matches.iter().map(|(_, _, lead)| *lead));

                let mut counted_alerts: BTreeSet<usize> = matches
                    .iter()
                    .map(|(_, alert_index, _)| *alert_index)
                    .collect();
                counted_alerts.extend(early_matches.iter().map(|(_, alert_index, _)| *alert_index));
                false_alert_count += alerts.len().saturating_sub(counted_alerts.len());
            }
        }
    }

    let event_recall_within_tolerance =
        (known_event_count > 0).then_some(matched_event_count as f64 / known_event_count as f64);
    let exact_month_event_recall = (known_event_count > 0)
        .then_some(exact_month_matched_event_count as f64 / known_event_count as f64);
    let early_warning_event_rate = (known_event_count > 0)
        .then_some(early_warning_event_count as f64 / known_event_count as f64);
    let observation_coverage = (expected_labelled_site_months > 0)
        .then_some(observed_labelled_site_months as f64 / expected_labelled_site_months as f64);
    let false_alerts_per_100_observed_site_months = (observed_labelled_site_months > 0)
        .then_some(false_alert_count as f64 * 100.0 / observed_labelled_site_months as f64);

    Ok(ThreatMonitoringReport {
        schema_version: THREAT_REPORT_SCHEMA_VERSION.to_owned(),
        site_series_count: series.len(),
        known_event_site_count,
        confirmed_no_event_site_count,
        unknown_site_count,
        known_event_count,
        matched_event_count,
        exact_month_matched_event_count,
        event_recall_within_tolerance,
        exact_month_event_recall,
        median_absolute_localisation_error_months: median_u64(&mut absolute_errors),
        early_warning_event_count,
        early_warning_event_rate,
        median_early_warning_lead_months: median_u64(&mut early_warning_leads),
        alert_episode_count,
        false_alert_count,
        expected_labelled_site_months,
        observed_labelled_site_months,
        observation_coverage,
        false_alerts_per_100_observed_site_months,
        alert_threshold: config.alert_threshold,
        temporal_tolerance_months: config.temporal_tolerance_months,
        alert_merge_gap_months: config.alert_merge_gap_months,
        early_warning_window_months: config.early_warning_window_months,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ThreatEvaluationConfig {
        ThreatEvaluationConfig {
            alert_threshold: 0.7,
            temporal_tolerance_months: 3,
            alert_merge_gap_months: 0,
            early_warning_window_months: 6,
        }
    }

    fn observation(month: i64, score: Option<f64>) -> ThreatMonthObservation {
        ThreatMonthObservation {
            month_ordinal: month,
            alert_score: score,
        }
    }

    fn known_series(
        site_id: &str,
        start: i64,
        end: i64,
        events: Vec<i64>,
        observations: Vec<ThreatMonthObservation>,
    ) -> ThreatMonitoringSeries {
        ThreatMonitoringSeries {
            schema_version: THREAT_SERIES_SCHEMA_VERSION.to_owned(),
            series_id: format!("series-{site_id}"),
            site_id: site_id.to_owned(),
            ground_truth: ThreatGroundTruth::KnownEvents {
                monitoring_start_month: start,
                monitoring_end_month: end,
                event_months: events,
            },
            observations,
        }
    }

    #[test]
    fn month_ordinal_round_trips_calendar_months() {
        let month = month_ordinal(2024, 2).unwrap();
        assert_eq!(year_month_from_ordinal(month), Some((2024, 2)));
        assert!(month_ordinal(2024, 0).is_none());
        assert!(month_ordinal(10000, 1).is_none());
    }

    #[test]
    fn exact_month_alert_is_recalled_with_zero_localisation_error() {
        let event = month_ordinal(2024, 3).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-a",
                event - 1,
                event + 1,
                vec![event],
                vec![
                    observation(event - 1, Some(0.1)),
                    observation(event, Some(0.95)),
                    observation(event + 1, Some(0.2)),
                ],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.known_event_count, 1);
        assert_eq!(report.matched_event_count, 1);
        assert_eq!(report.exact_month_matched_event_count, 1);
        assert_eq!(report.event_recall_within_tolerance, Some(1.0));
        assert_eq!(report.exact_month_event_recall, Some(1.0));
        assert_eq!(report.median_absolute_localisation_error_months, Some(0.0));
        assert_eq!(report.false_alert_count, 0);
    }

    #[test]
    fn tolerance_and_early_warning_are_reported_separately() {
        let event = month_ordinal(2024, 7).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-b",
                event - 6,
                event + 1,
                vec![event],
                vec![
                    observation(event - 3, Some(0.85)),
                    observation(event - 2, Some(0.1)),
                    observation(event, Some(0.2)),
                ],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.event_recall_within_tolerance, Some(1.0));
        assert_eq!(report.exact_month_event_recall, Some(0.0));
        assert_eq!(report.early_warning_event_count, 1);
        assert_eq!(report.median_early_warning_lead_months, Some(3.0));
    }

    #[test]
    fn early_warning_outside_event_tolerance_is_not_a_false_alert() {
        let event = month_ordinal(2024, 7).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-early-only",
                event - 6,
                event,
                vec![event],
                vec![observation(event - 6, Some(0.9))],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.event_recall_within_tolerance, Some(0.0));
        assert_eq!(report.early_warning_event_count, 1);
        assert_eq!(report.median_early_warning_lead_months, Some(6.0));
        assert_eq!(report.false_alert_count, 0);
    }

    #[test]
    fn repeated_events_are_matched_one_to_one() {
        let event_a = month_ordinal(2023, 2).unwrap();
        let event_b = month_ordinal(2023, 8).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-c",
                event_a - 1,
                event_b + 1,
                vec![event_a, event_b],
                vec![
                    observation(event_a - 1, Some(0.9)),
                    observation(event_a + 1, Some(0.1)),
                    observation(event_b - 1, Some(0.8)),
                    observation(event_b + 1, Some(0.1)),
                ],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.known_event_count, 2);
        assert_eq!(report.matched_event_count, 2);
        assert_eq!(report.event_recall_within_tolerance, Some(1.0));
        assert_eq!(report.early_warning_event_count, 2);
        assert_eq!(report.median_early_warning_lead_months, Some(1.0));
    }

    #[test]
    fn consecutive_alert_months_merge_into_one_episode() {
        let event = month_ordinal(2023, 5).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-d",
                event - 2,
                event + 1,
                vec![event],
                vec![
                    observation(event - 1, Some(0.8)),
                    observation(event, Some(0.9)),
                    observation(event + 1, Some(0.1)),
                ],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.alert_episode_count, 1);
        assert_eq!(report.matched_event_count, 1);
        assert_eq!(report.exact_month_matched_event_count, 1);
    }

    #[test]
    fn unknown_sites_never_become_negative_controls() {
        let start = month_ordinal(2023, 1).unwrap();
        let end = month_ordinal(2023, 3).unwrap();
        let unknown = ThreatMonitoringSeries {
            schema_version: THREAT_SERIES_SCHEMA_VERSION.to_owned(),
            series_id: "series-unknown".to_owned(),
            site_id: "site-unknown".to_owned(),
            ground_truth: ThreatGroundTruth::Unknown,
            observations: vec![
                observation(start, Some(0.99)),
                observation(start + 1, Some(0.1)),
                observation(end, Some(0.95)),
            ],
        };
        let no_event = ThreatMonitoringSeries {
            schema_version: THREAT_SERIES_SCHEMA_VERSION.to_owned(),
            series_id: "series-confirmed-safe".to_owned(),
            site_id: "site-confirmed-safe".to_owned(),
            ground_truth: ThreatGroundTruth::ConfirmedNoEventForWindow {
                monitoring_start_month: start,
                monitoring_end_month: end,
            },
            observations: vec![
                observation(start, Some(0.99)),
                observation(start + 1, None),
                observation(end, Some(0.1)),
            ],
        };

        let report = evaluate_threat_monitoring(&[unknown, no_event], config()).unwrap();
        assert_eq!(report.unknown_site_count, 1);
        assert_eq!(report.confirmed_no_event_site_count, 1);
        assert_eq!(report.alert_episode_count, 1);
        assert_eq!(report.false_alert_count, 1);
        assert_eq!(report.expected_labelled_site_months, 3);
        assert_eq!(report.observed_labelled_site_months, 2);
        assert_eq!(report.observation_coverage, Some(2.0 / 3.0));
    }

    #[test]
    fn missing_months_lower_coverage_but_are_not_false_alerts_or_negatives() {
        let event = month_ordinal(2023, 6).unwrap();
        let report = evaluate_threat_monitoring(
            &[known_series(
                "site-e",
                event - 5,
                event + 4,
                vec![event],
                vec![
                    observation(event - 2, Some(0.9)),
                    observation(event, None),
                    observation(event + 2, Some(0.1)),
                ],
            )],
            config(),
        )
        .unwrap();

        assert_eq!(report.expected_labelled_site_months, 10);
        assert_eq!(report.observed_labelled_site_months, 2);
        assert_eq!(report.observation_coverage, Some(0.2));
        assert_eq!(report.matched_event_count, 1);
        assert_eq!(report.false_alert_count, 0);
    }

    #[test]
    fn false_alert_rate_uses_only_verified_labelled_months() {
        let start = month_ordinal(2023, 1).unwrap();
        let end = month_ordinal(2023, 4).unwrap();
        let no_event = ThreatMonitoringSeries {
            schema_version: THREAT_SERIES_SCHEMA_VERSION.to_owned(),
            series_id: "series-control".to_owned(),
            site_id: "site-control".to_owned(),
            ground_truth: ThreatGroundTruth::ConfirmedNoEventForWindow {
                monitoring_start_month: start,
                monitoring_end_month: end,
            },
            observations: vec![
                observation(start, Some(0.8)),
                observation(start + 1, Some(0.1)),
                observation(start + 2, Some(0.9)),
                observation(end, Some(0.2)),
            ],
        };
        let report = evaluate_threat_monitoring(&[no_event], config()).unwrap();
        assert_eq!(report.false_alert_count, 2);
        assert_eq!(report.false_alerts_per_100_observed_site_months, Some(50.0));
    }

    #[test]
    fn invalid_scores_duplicate_months_and_duplicate_sites_fail_closed() {
        let month = month_ordinal(2023, 1).unwrap();
        let mut bad_score = known_series(
            "site-invalid",
            month,
            month,
            vec![month],
            vec![observation(month, Some(f64::NAN))],
        );
        assert!(matches!(
            evaluate_threat_monitoring(&[bad_score.clone()], config()),
            Err(ThreatEvaluationError::InvalidAlertScore { .. })
        ));

        bad_score.observations = vec![observation(month, Some(0.1)), observation(month, Some(0.2))];
        assert!(matches!(
            evaluate_threat_monitoring(&[bad_score], config()),
            Err(ThreatEvaluationError::ObservationsNotStrictlyIncreasing { .. })
        ));

        let good = known_series(
            "site-duplicate",
            month,
            month,
            vec![month],
            vec![observation(month, Some(0.9))],
        );
        assert_eq!(
            evaluate_threat_monitoring(&[good.clone(), good], config()),
            Err(ThreatEvaluationError::DuplicateSiteId(
                "site-duplicate".to_owned()
            ))
        );
    }

    #[test]
    fn invalid_or_ambiguous_ground_truth_fails_closed() {
        let month = month_ordinal(2023, 1).unwrap();
        let mut empty_events = known_series("site-empty", month, month + 1, Vec::new(), Vec::new());
        assert!(matches!(
            evaluate_threat_monitoring(&[empty_events.clone()], config()),
            Err(ThreatEvaluationError::KnownEventsListEmpty { .. })
        ));

        empty_events.ground_truth = ThreatGroundTruth::KnownEvents {
            monitoring_start_month: month + 1,
            monitoring_end_month: month,
            event_months: vec![month],
        };
        assert!(matches!(
            evaluate_threat_monitoring(&[empty_events], config()),
            Err(ThreatEvaluationError::InvalidMonitoringWindow { .. })
        ));

        let invalid_config = ThreatEvaluationConfig {
            alert_threshold: 1.1,
            ..config()
        };
        assert_eq!(
            evaluate_threat_monitoring(&[], invalid_config),
            Err(ThreatEvaluationError::InvalidConfig)
        );
    }

    #[test]
    fn zero_event_denominator_and_zero_observation_denominator_are_explicit() {
        let start = month_ordinal(2023, 1).unwrap();
        let no_event = ThreatMonitoringSeries {
            schema_version: THREAT_SERIES_SCHEMA_VERSION.to_owned(),
            series_id: "series-no-observations".to_owned(),
            site_id: "site-no-observations".to_owned(),
            ground_truth: ThreatGroundTruth::ConfirmedNoEventForWindow {
                monitoring_start_month: start,
                monitoring_end_month: start + 1,
            },
            observations: vec![observation(start, None)],
        };
        let report = evaluate_threat_monitoring(&[no_event], config()).unwrap();
        assert_eq!(report.event_recall_within_tolerance, None);
        assert_eq!(report.observation_coverage, Some(0.0));
        assert_eq!(report.false_alerts_per_100_observed_site_months, None);
    }
}
