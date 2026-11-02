// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Interval-censored event-time evaluation for archaeological monitoring.
//!
//! The event time is often only known to lie between a last-unaffected and a
//! first-affected observation. This evaluator scores a probability
//! distribution against that interval rather than manufacturing a single
//! ground-truth month. It also evaluates partial ordering only when truth
//! intervals do not overlap. This module does not detect change, attribute
//! cause, or decide heritage significance.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::heritage_threat::year_month_from_ordinal;

pub const TEMPORAL_INTERVAL_SCHEMA_VERSION: &str =
    "luminous.heritage-temporal-interval.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventTimeInterval {
    /// Inclusive first month in which the event could have occurred.
    pub start_month: i64,
    /// Inclusive last month in which the event could have occurred.
    pub end_month: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonthProbability {
    pub month_ordinal: i64,
    /// Conditional probability of the event occurring in this month.
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalEventPrediction {
    pub event_id: String,
    /// Strictly increasing month ordinals with a normalized conditional
    /// probability distribution. Missing imagery months are not implicit
    /// no-event labels; the model may assign probability to unobserved months.
    pub month_probabilities: Vec<MonthProbability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalEventTruth {
    pub event_id: String,
    pub admissible_interval: EventTimeInterval,
    pub adjudication_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalTimelineCase {
    pub case_id: String,
    pub candidate_id: String,
    pub predictions: Vec<TemporalEventPrediction>,
    pub ground_truth: Vec<TemporalEventTruth>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemporalIntervalConfig {
    /// Number of highest-probability months used for Recall@K.
    pub top_k_months: usize,
    pub probability_tolerance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalEventMetrics {
    pub event_id: String,
    pub admissible_start_month: i64,
    pub admissible_end_month: i64,
    pub map_month: i64,
    pub map_inside_interval: bool,
    pub top_k_intersects_interval: bool,
    pub probability_mass_in_interval: f64,
    pub negative_log_interval_likelihood: f64,
    /// Zero when MAP lies inside the admissible interval.
    pub map_distance_to_interval_months: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalIntervalReport {
    pub schema_version: String,
    pub case_id: String,
    pub candidate_id: String,
    pub event_count: usize,
    pub map_inside_interval_count: usize,
    pub map_inside_interval_rate: f64,
    pub top_k_months: usize,
    pub recall_at_k_count: usize,
    pub recall_at_k: f64,
    pub mean_probability_mass_in_interval: f64,
    pub mean_negative_log_interval_likelihood: f64,
    pub mean_map_distance_to_interval_months: f64,
    pub ordered_event_pair_count: usize,
    pub ambiguous_event_pair_count: usize,
    pub decisive_order_pair_count: usize,
    pub tied_order_pair_count: usize,
    pub pairwise_order_accuracy: Option<f64>,
    pub mean_pairwise_order_brier: Option<f64>,
    pub events: Vec<TemporalEventMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalIntervalError {
    EmptyCaseId,
    EmptyCandidateId,
    InvalidConfig,
    EmptyPredictions,
    EmptyGroundTruth,
    EmptyEventId,
    DuplicatePredictionEventId(String),
    DuplicateGroundTruthEventId(String),
    MissingPredictionForEvent(String),
    PredictionWithoutGroundTruth(String),
    InvalidEventInterval { event_id: String },
    EmptyAdjudicationReference { event_id: String },
    EmptyEvidenceReferences { event_id: String },
    InvalidEvidenceReference { event_id: String },
    DuplicateEvidenceReference { event_id: String },
    EmptyMonthProbabilities { event_id: String },
    InvalidMonthOrdinal { event_id: String, month_ordinal: i64 },
    MonthProbabilitiesNotStrictlyIncreasing { event_id: String },
    InvalidMonthProbability { event_id: String, month_ordinal: i64 },
    MonthProbabilitySum { event_id: String, sum_millionths: i64 },
    ZeroProbabilityInAdmissibleInterval { event_id: String },
    InvalidIntervalDistance { event_id: String },
}

impl fmt::Display for TemporalIntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage temporal interval error: {self:?}")
    }
}

impl std::error::Error for TemporalIntervalError {}

fn valid_interval(interval: EventTimeInterval) -> bool {
    interval.start_month <= interval.end_month
        && year_month_from_ordinal(interval.start_month).is_some()
        && year_month_from_ordinal(interval.end_month).is_some()
}

fn interval_distance(
    month: i64,
    interval: EventTimeInterval,
    event_id: &str,
) -> Result<u64, TemporalIntervalError> {
    let distance = if month < interval.start_month {
        interval.start_month.checked_sub(month)
    } else if month > interval.end_month {
        month.checked_sub(interval.end_month)
    } else {
        Some(0)
    };
    distance
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| TemporalIntervalError::InvalidIntervalDistance {
            event_id: event_id.to_owned(),
        })
}

fn map_month(probabilities: &[MonthProbability]) -> i64 {
    // The input is ordered by month. Updating only on a strict increase makes
    // the earliest month win exact probability ties deterministically.
    let mut best_month = probabilities[0].month_ordinal;
    let mut best_probability = probabilities[0].probability;
    for item in probabilities.iter().skip(1) {
        if item.probability > best_probability {
            best_probability = item.probability;
            best_month = item.month_ordinal;
        }
    }
    best_month
}

fn validate_prediction(
    prediction: &TemporalEventPrediction,
    tolerance: f64,
) -> Result<(), TemporalIntervalError> {
    if prediction.event_id.trim().is_empty() {
        return Err(TemporalIntervalError::EmptyEventId);
    }
    if prediction.month_probabilities.is_empty() {
        return Err(TemporalIntervalError::EmptyMonthProbabilities {
            event_id: prediction.event_id.clone(),
        });
    }

    let mut sum = 0.0;
    let mut previous = None;
    for item in &prediction.month_probabilities {
        if year_month_from_ordinal(item.month_ordinal).is_none() {
            return Err(TemporalIntervalError::InvalidMonthOrdinal {
                event_id: prediction.event_id.clone(),
                month_ordinal: item.month_ordinal,
            });
        }
        if previous.is_some_and(|month| item.month_ordinal <= month) {
            return Err(
                TemporalIntervalError::MonthProbabilitiesNotStrictlyIncreasing {
                    event_id: prediction.event_id.clone(),
                },
            );
        }
        if !item.probability.is_finite() || !(0.0..=1.0).contains(&item.probability) {
            return Err(TemporalIntervalError::InvalidMonthProbability {
                event_id: prediction.event_id.clone(),
                month_ordinal: item.month_ordinal,
            });
        }
        previous = Some(item.month_ordinal);
        sum += item.probability;
    }
    if !sum.is_finite() || (sum - 1.0).abs() > tolerance {
        return Err(TemporalIntervalError::MonthProbabilitySum {
            event_id: prediction.event_id.clone(),
            sum_millionths: (sum * 1_000_000.0).round() as i64,
        });
    }
    Ok(())
}

fn validate_truth(truth: &TemporalEventTruth) -> Result<(), TemporalIntervalError> {
    if truth.event_id.trim().is_empty() {
        return Err(TemporalIntervalError::EmptyEventId);
    }
    if !valid_interval(truth.admissible_interval) {
        return Err(TemporalIntervalError::InvalidEventInterval {
            event_id: truth.event_id.clone(),
        });
    }
    if truth.adjudication_ref.trim().is_empty() {
        return Err(TemporalIntervalError::EmptyAdjudicationReference {
            event_id: truth.event_id.clone(),
        });
    }
    if truth.evidence_refs.is_empty() {
        return Err(TemporalIntervalError::EmptyEvidenceReferences {
            event_id: truth.event_id.clone(),
        });
    }
    if truth.evidence_refs.iter().any(|reference| reference.trim().is_empty()) {
        return Err(TemporalIntervalError::InvalidEvidenceReference {
            event_id: truth.event_id.clone(),
        });
    }
    let unique_references = truth.evidence_refs.iter().collect::<BTreeSet<_>>();
    if unique_references.len() != truth.evidence_refs.len() {
        return Err(TemporalIntervalError::DuplicateEvidenceReference {
            event_id: truth.event_id.clone(),
        });
    }
    Ok(())
}

fn probability_first_before_second(
    first: &[MonthProbability],
    second: &[MonthProbability],
) -> f64 {
    let mut probability = 0.0;
    for first_month in first {
        for second_month in second {
            if first_month.month_ordinal < second_month.month_ordinal {
                probability += first_month.probability * second_month.probability;
            }
        }
    }
    probability.clamp(0.0, 1.0)
}

/// Evaluate one candidate's event-time posterior distributions against
/// evidence-supported admissible intervals and the partial order implied by
/// non-overlapping event intervals.
///
/// This scores localization conditional on the event being known to have
/// happened. Detection precision/recall and event-cause classification are
/// separate tasks and are deliberately not inferred here.
pub fn evaluate_temporal_intervals(
    case: &TemporalTimelineCase,
    config: TemporalIntervalConfig,
) -> Result<TemporalIntervalReport, TemporalIntervalError> {
    if case.case_id.trim().is_empty() {
        return Err(TemporalIntervalError::EmptyCaseId);
    }
    if case.candidate_id.trim().is_empty() {
        return Err(TemporalIntervalError::EmptyCandidateId);
    }
    if config.top_k_months == 0
        || !config.probability_tolerance.is_finite()
        || config.probability_tolerance <= 0.0
        || config.probability_tolerance > 1e-6
    {
        return Err(TemporalIntervalError::InvalidConfig);
    }
    if case.predictions.is_empty() {
        return Err(TemporalIntervalError::EmptyPredictions);
    }
    if case.ground_truth.is_empty() {
        return Err(TemporalIntervalError::EmptyGroundTruth);
    }

    let mut prediction_ids = BTreeSet::new();
    let mut predictions_by_id = BTreeMap::new();
    for prediction in &case.predictions {
        if prediction.event_id.trim().is_empty() {
            return Err(TemporalIntervalError::EmptyEventId);
        }
        if !prediction_ids.insert(prediction.event_id.as_str()) {
            return Err(TemporalIntervalError::DuplicatePredictionEventId(
                prediction.event_id.clone(),
            ));
        }
        validate_prediction(prediction, config.probability_tolerance)?;
        predictions_by_id.insert(prediction.event_id.as_str(), prediction);
    }

    let mut truth_ids = BTreeSet::new();
    for truth in &case.ground_truth {
        validate_truth(truth)?;
        if !truth_ids.insert(truth.event_id.as_str()) {
            return Err(TemporalIntervalError::DuplicateGroundTruthEventId(
                truth.event_id.clone(),
            ));
        }
        if !predictions_by_id.contains_key(truth.event_id.as_str()) {
            return Err(TemporalIntervalError::MissingPredictionForEvent(
                truth.event_id.clone(),
            ));
        }
    }
    for prediction in &case.predictions {
        if !truth_ids.contains(prediction.event_id.as_str()) {
            return Err(TemporalIntervalError::PredictionWithoutGroundTruth(
                prediction.event_id.clone(),
            ));
        }
    }

    let mut ordered_truth = case.ground_truth.iter().collect::<Vec<_>>();
    ordered_truth.sort_by(|left, right| {
        left.admissible_interval
            .start_month
            .cmp(&right.admissible_interval.start_month)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });

    let mut events = Vec::with_capacity(ordered_truth.len());
    let mut map_inside_count = 0usize;
    let mut recall_at_k_count = 0usize;
    let mut probability_mass_sum = 0.0;
    let mut interval_nll_sum = 0.0;
    let mut map_distance_sum = 0.0;

    for truth in &ordered_truth {
        let prediction = predictions_by_id
            .get(truth.event_id.as_str())
            .expect("prediction membership was validated");
        let interval = truth.admissible_interval;
        let probabilities = &prediction.month_probabilities;
        let map = map_month(probabilities);
        let map_inside = map >= interval.start_month && map <= interval.end_month;
        if map_inside {
            map_inside_count += 1;
        }

        let mut mass = 0.0;
        for item in probabilities {
            if (interval.start_month..=interval.end_month).contains(&item.month_ordinal) {
                mass += item.probability;
            }
        }
        if !mass.is_finite() || mass <= 0.0 {
            return Err(TemporalIntervalError::ZeroProbabilityInAdmissibleInterval {
                event_id: truth.event_id.clone(),
            });
        }
        // A normalized distribution may sum to 1 within tolerance with tiny
        // floating-point excess; interval probability cannot exceed 1.
        mass = mass.min(1.0);
        let nll = -mass.ln();

        let mut ranked = probabilities.iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .probability
                .total_cmp(&left.probability)
                .then_with(|| left.month_ordinal.cmp(&right.month_ordinal))
        });
        let top_k_hit = ranked
            .iter()
            .take(config.top_k_months)
            .any(|item| (interval.start_month..=interval.end_month).contains(&item.month_ordinal));
        if top_k_hit {
            recall_at_k_count += 1;
        }

        let distance = interval_distance(map, interval, &truth.event_id)?;

        probability_mass_sum += mass;
        interval_nll_sum += nll;
        map_distance_sum += distance as f64;
        events.push(TemporalEventMetrics {
            event_id: truth.event_id.clone(),
            admissible_start_month: interval.start_month,
            admissible_end_month: interval.end_month,
            map_month: map,
            map_inside_interval: map_inside,
            top_k_intersects_interval: top_k_hit,
            probability_mass_in_interval: mass,
            negative_log_interval_likelihood: nll,
            map_distance_to_interval_months: distance,
        });
    }

    let mut ordered_event_pair_count = 0usize;
    let mut ambiguous_event_pair_count = 0usize;
    let mut decisive_order_pair_count = 0usize;
    let mut tied_order_pair_count = 0usize;
    let mut order_correct_count = 0usize;
    let mut order_brier_sum = 0.0;

    for left_index in 0..ordered_truth.len() {
        for right_index in (left_index + 1)..ordered_truth.len() {
            let left = ordered_truth[left_index];
            let right = ordered_truth[right_index];
            let left_interval = left.admissible_interval;
            let right_interval = right.admissible_interval;

            let truth_left_before = left_interval.end_month < right_interval.start_month;
            let truth_right_before = right_interval.end_month < left_interval.start_month;
            if !truth_left_before && !truth_right_before {
                ambiguous_event_pair_count += 1;
                continue;
            }

            ordered_event_pair_count += 1;
            let left_prediction = predictions_by_id
                .get(left.event_id.as_str())
                .expect("prediction membership was validated");
            let right_prediction = predictions_by_id
                .get(right.event_id.as_str())
                .expect("prediction membership was validated");
            let p_left_before = probability_first_before_second(
                &left_prediction.month_probabilities,
                &right_prediction.month_probabilities,
            );
            let observed = if truth_left_before { 1.0 } else { 0.0 };
            order_brier_sum += (p_left_before - observed).powi(2);

            if (p_left_before - 0.5).abs() <= config.probability_tolerance {
                tied_order_pair_count += 1;
            } else {
                decisive_order_pair_count += 1;
                let predicted_left_before = p_left_before > 0.5;
                if predicted_left_before == truth_left_before {
                    order_correct_count += 1;
                }
            }
        }
    }

    let event_count = events.len();
    let ordered_pair_denominator = ordered_event_pair_count as f64;
    Ok(TemporalIntervalReport {
        schema_version: TEMPORAL_INTERVAL_SCHEMA_VERSION.to_owned(),
        case_id: case.case_id.clone(),
        candidate_id: case.candidate_id.clone(),
        event_count,
        map_inside_interval_count: map_inside_count,
        map_inside_interval_rate: map_inside_count as f64 / event_count as f64,
        top_k_months: config.top_k_months,
        recall_at_k_count,
        recall_at_k: recall_at_k_count as f64 / event_count as f64,
        mean_probability_mass_in_interval: probability_mass_sum / event_count as f64,
        mean_negative_log_interval_likelihood: interval_nll_sum / event_count as f64,
        mean_map_distance_to_interval_months: map_distance_sum / event_count as f64,
        ordered_event_pair_count,
        ambiguous_event_pair_count,
        decisive_order_pair_count,
        tied_order_pair_count,
        pairwise_order_accuracy: (decisive_order_pair_count > 0)
            .then_some(order_correct_count as f64 / decisive_order_pair_count as f64),
        mean_pairwise_order_brier: (ordered_event_pair_count > 0)
            .then_some(order_brier_sum / ordered_pair_denominator),
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::heritage_threat::month_ordinal;

    fn month(year: i32, month: u8) -> i64 {
        month_ordinal(year, month).unwrap()
    }

    fn prediction(
        event_id: &str,
        months: &[(i32, u8, f64)],
    ) -> TemporalEventPrediction {
        TemporalEventPrediction {
            event_id: event_id.into(),
            month_probabilities: months
                .iter()
                .map(|(year, month, probability)| MonthProbability {
                    month_ordinal: month(*year, *month),
                    probability: *probability,
                })
                .collect(),
        }
    }

    fn truth(event_id: &str, start: (i32, u8), end: (i32, u8)) -> TemporalEventTruth {
        TemporalEventTruth {
            event_id: event_id.into(),
            admissible_interval: EventTimeInterval {
                start_month: month(start.0, start.1),
                end_month: month(end.0, end.1),
            },
            adjudication_ref: format!("urn:synthetic:adjudication:{event_id}"),
            evidence_refs: vec![format!("urn:synthetic:evidence:{event_id}")],
        }
    }

    fn config(top_k: usize) -> TemporalIntervalConfig {
        TemporalIntervalConfig {
            top_k_months: top_k,
            probability_tolerance: 1e-9,
        }
    }

    fn case(
        predictions: Vec<TemporalEventPrediction>,
        ground_truth: Vec<TemporalEventTruth>,
    ) -> TemporalTimelineCase {
        TemporalTimelineCase {
            case_id: "synthetic-case".into(),
            candidate_id: "synthetic-candidate".into(),
            predictions,
            ground_truth,
        }
    }

    #[test]
    fn interval_mass_and_map_hit_are_scored_without_forcing_an_exact_month() {
        let input = case(
            vec![prediction(
                "event-a",
                &[(2020, 1, 0.2), (2020, 2, 0.5), (2020, 3, 0.3)],
            )],
            vec![truth("event-a", (2020, 2), (2020, 3))],
        );
        let report = evaluate_temporal_intervals(&input, config(2)).unwrap();
        assert_eq!(report.map_inside_interval_count, 1);
        assert_eq!(report.recall_at_k, 1.0);
        assert!((report.mean_probability_mass_in_interval - 0.8).abs() < 1e-12);
        assert!((report.mean_negative_log_interval_likelihood - -0.8_f64.ln()).abs() < 1e-12);
        assert_eq!(report.mean_map_distance_to_interval_months, 0.0);
    }

    #[test]
    fn top_k_can_recover_an_event_when_map_month_is_outside_its_interval() {
        let input = case(
            vec![prediction(
                "event-a",
                &[(2020, 1, 0.45), (2020, 2, 0.35), (2020, 3, 0.2)],
            )],
            vec![truth("event-a", (2020, 2), (2020, 3))],
        );
        let report = evaluate_temporal_intervals(&input, config(2)).unwrap();
        assert_eq!(report.map_inside_interval_count, 0);
        assert_eq!(report.recall_at_k, 1.0);
        assert_eq!(report.events[0].map_distance_to_interval_months, 1);
    }

    #[test]
    fn zero_probability_in_the_admissible_interval_fails_closed() {
        let input = case(
            vec![prediction(
                "event-a",
                &[(2020, 1, 0.6), (2020, 2, 0.4)],
            )],
            vec![truth("event-a", (2020, 3), (2020, 4))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&input, config(1)),
            Err(TemporalIntervalError::ZeroProbabilityInAdmissibleInterval { .. })
        ));
    }

    #[test]
    fn overlapping_event_intervals_are_ambiguous_not_forced_into_order() {
        let input = case(
            vec![
                prediction("event-a", &[(2020, 1, 1.0)]),
                prediction("event-b", &[(2020, 3, 1.0)]),
            ],
            vec![
                truth("event-a", (2020, 1), (2020, 3)),
                truth("event-b", (2020, 2), (2020, 4)),
            ],
        );
        let report = evaluate_temporal_intervals(&input, config(1)).unwrap();
        assert_eq!(report.ordered_event_pair_count, 0);
        assert_eq!(report.ambiguous_event_pair_count, 1);
        assert!(report.pairwise_order_accuracy.is_none());
        assert!(report.mean_pairwise_order_brier.is_none());
    }

    #[test]
    fn disjoint_event_intervals_get_a_pairwise_order_score() {
        let input = case(
            vec![
                prediction("event-a", &[(2020, 1, 0.8), (2020, 2, 0.2)]),
                prediction("event-b", &[(2020, 4, 0.1), (2020, 5, 0.9)]),
            ],
            vec![
                truth("event-a", (2020, 1), (2020, 2)),
                truth("event-b", (2020, 4), (2020, 5)),
            ],
        );
        let report = evaluate_temporal_intervals(&input, config(1)).unwrap();
        assert_eq!(report.ordered_event_pair_count, 1);
        assert_eq!(report.ambiguous_event_pair_count, 0);
        assert_eq!(report.pairwise_order_accuracy, Some(1.0));
        assert_eq!(report.mean_pairwise_order_brier, Some(0.0));
    }

    #[test]
    fn pairwise_order_brier_penalizes_overlapping_temporal_posteriors() {
        let input = case(
            vec![
                prediction("event-a", &[(2020, 1, 0.5), (2020, 4, 0.5)]),
                prediction("event-b", &[(2020, 2, 0.8), (2020, 5, 0.2)]),
            ],
            vec![
                truth("event-a", (2020, 1), (2020, 1)),
                truth("event-b", (2020, 5), (2020, 5)),
            ],
        );
        let report = evaluate_temporal_intervals(&input, config(1)).unwrap();
        assert_eq!(report.ordered_event_pair_count, 1);
        assert!((report.mean_pairwise_order_brier.unwrap() - 0.16).abs() < 1e-12);
    }

    #[test]
    fn probability_ties_choose_the_earliest_month_and_top_k_ties_are_stable() {
        let input = case(
            vec![prediction(
                "event-a",
                &[(2020, 1, 0.5), (2020, 2, 0.5)],
            )],
            vec![truth("event-a", (2020, 2), (2020, 2))],
        );
        let report = evaluate_temporal_intervals(&input, config(1)).unwrap();
        assert_eq!(report.events[0].map_month, month(2020, 1));
        assert!(!report.events[0].top_k_intersects_interval);
    }

    #[test]
    fn normalized_distribution_and_interval_are_required() {
        let mut input = case(
            vec![prediction(
                "event-a",
                &[(2020, 1, 0.4), (2020, 2, 0.4)],
            )],
            vec![truth("event-a", (2020, 1), (2020, 2))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&input, config(1)),
            Err(TemporalIntervalError::MonthProbabilitySum { .. })
        ));
        input.ground_truth[0].admissible_interval.end_month =
            month(2019, 1);
        input.predictions[0].month_probabilities[0].probability = 0.6;
        assert!(matches!(
            evaluate_temporal_intervals(&input, config(1)),
            Err(TemporalIntervalError::InvalidEventInterval { .. })
        ));
    }

    #[test]
    fn duplicate_missing_and_extra_event_ids_fail_closed() {
        let duplicate = case(
            vec![
                prediction("event-a", &[(2020, 1, 1.0)]),
                prediction("event-a", &[(2020, 1, 1.0)]),
            ],
            vec![truth("event-a", (2020, 1), (2020, 1))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&duplicate, config(1)),
            Err(TemporalIntervalError::DuplicatePredictionEventId(_))
        ));

        let missing = case(
            vec![prediction("event-b", &[(2020, 1, 1.0)])],
            vec![truth("event-a", (2020, 1), (2020, 1))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&missing, config(1)),
            Err(TemporalIntervalError::MissingPredictionForEvent(_))
        ));

        let extra = case(
            vec![
                prediction("event-a", &[(2020, 1, 1.0)]),
                prediction("event-b", &[(2020, 2, 1.0)]),
            ],
            vec![truth("event-a", (2020, 1), (2020, 1))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&extra, config(1)),
            Err(TemporalIntervalError::PredictionWithoutGroundTruth(_))
        ));
    }

    #[test]
    fn duplicate_evidence_references_fail_closed() {
        let mut input = case(
            vec![prediction("event-a", &[(2020, 1, 1.0)])],
            vec![truth("event-a", (2020, 1), (2020, 1))],
        );
        input.ground_truth[0].evidence_refs.push(
            input.ground_truth[0].evidence_refs[0].clone()
        );
        assert!(matches!(
            evaluate_temporal_intervals(&input, config(1)),
            Err(TemporalIntervalError::DuplicateEvidenceReference { .. })
        ));
    }

    #[test]
    fn invalid_month_probabilities_and_config_fail_closed() {
        let invalid = case(
            vec![prediction("event-a", &[(2020, 1, f64::NAN)])],
            vec![truth("event-a", (2020, 1), (2020, 1))],
        );
        assert!(matches!(
            evaluate_temporal_intervals(&invalid, config(1)),
            Err(TemporalIntervalError::InvalidMonthProbability { .. })
        ));
        assert_eq!(
            evaluate_temporal_intervals(
                &invalid,
                TemporalIntervalConfig { top_k_months: 0, probability_tolerance: 1e-9 }
            ),
            Err(TemporalIntervalError::InvalidConfig)
        );
    }
}
