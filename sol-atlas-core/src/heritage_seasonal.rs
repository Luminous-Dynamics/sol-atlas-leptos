// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fixed-reference seasonal robust deviation for archaeological change monitoring.
//!
//! Each evaluation month is compared only with observations from the frozen
//! historical reference interval and a circular month-of-year window. Missing
//! values remain unobserved; an inadequate number of distinct reference years
//! produces abstention, never a zero anomaly. Robust z-scores are not
//! probabilities. Optional two-sided CUSUM operates only on valid SRD scores
//! and has an explicit missing-month reset policy.
//!
//! This module detects statistical deviations in a declared feature. It does
//! not establish archaeological change, threat, cause or heritage significance.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const SEASONAL_SRD_SCHEMA_VERSION: &str = "luminous.heritage-seasonal-srd.v0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonalFeatureMetadata {
    pub series_id: String,
    pub site_id: String,
    /// Stable feature identity, such as ndvi, bsi, or a named embedding component.
    pub feature_id: String,
    pub feature_units: String,
    /// Explicit input semantics; never infer NIR from an arbitrary fourth channel.
    pub channel_semantics: String,
    pub feature_definition_ref: String,
    pub source_ref: String,
    /// SHA-256 of the exact source asset bytes, lowercase or uppercase hexadecimal.
    pub source_asset_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeasonalObservation {
    /// Integer month ordinal: year * 12 + (month - 1), for Gregorian years 1..=9999.
    pub month_ordinal: i64,
    /// None means no valid observation, not a zero-valued feature.
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonalInterval {
    pub start_month: i64,
    pub end_month: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeasonalCusumConfig {
    /// Non-negative drift allowance subtracted from each signed robust residual.
    pub drift_allowance: f64,
    /// Positive threshold applied independently to positive and negative CUSUM.
    pub alert_threshold: f64,
    /// Minimum valid SRD residuals in the current segment before an alert is allowed.
    pub min_valid_observations: usize,
    /// Reset after this many consecutive missing/unscored evaluation months.
    /// A missing-month count strictly greater than this value resets both sums.
    pub reset_after_missing_months: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalSrdConfig {
    /// Frozen historical interval. It must end strictly before evaluation starts.
    pub reference_interval: SeasonalInterval,
    pub evaluation_interval: SeasonalInterval,
    /// Circular month-of-year radius. 0 = same month; 1 includes adjacent months.
    /// A maximum of 6 avoids selecting the whole calendar twice.
    pub month_window: u8,
    /// Minimum number of distinct Gregorian years represented in the reference sample.
    pub min_reference_years: usize,
    /// Positive lower bound on the robust scale when MAD is zero or very small.
    pub scale_floor: f64,
    /// Absolute robust-z threshold. This is a thresholded score, not a probability.
    pub alert_threshold_abs: f64,
    /// Immutable policy/manifest reference for how this threshold was selected.
    pub alert_threshold_policy_ref: String,
    /// Optional persistent-shift CUSUM. Reported separately from single-month SRD.
    pub cusum: Option<SeasonalCusumConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalSrdSeries {
    pub metadata: SeasonalFeatureMetadata,
    /// Strictly increasing, unique month ordinals. Missing months need not be materialized.
    pub observations: Vec<SeasonalObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeasonalScoreState {
    Scored,
    /// No valid current observation exists for this month.
    Unobserved,
    /// A current observation exists, but the frozen reference lacks enough distinct years.
    InsufficientReference {
        observed_reference_years: usize,
        required_reference_years: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalSrdPoint {
    pub month_ordinal: i64,
    pub state: SeasonalScoreState,
    pub observation_value: Option<f64>,
    pub reference_observation_count: usize,
    pub reference_year_count: usize,
    pub reference_median: Option<f64>,
    pub reference_mad: Option<f64>,
    pub robust_scale: Option<f64>,
    pub signed_residual: Option<f64>,
    pub robust_z_score: Option<f64>,
    pub single_month_alert: Option<bool>,
    pub cusum_positive: Option<f64>,
    pub cusum_negative: Option<f64>,
    pub cusum_valid_observation_count: Option<usize>,
    pub cusum_reset_applied: Option<bool>,
    pub cusum_alert: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalSrdReport {
    pub schema_version: String,
    pub metadata: SeasonalFeatureMetadata,
    pub config: SeasonalSrdConfig,
    pub scored_month_count: usize,
    pub unobserved_month_count: usize,
    pub insufficient_reference_month_count: usize,
    pub single_month_alert_count: usize,
    pub cusum_alert_count: usize,
    pub points: Vec<SeasonalSrdPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeasonalSrdError {
    EmptySeriesId,
    EmptySiteId,
    EmptyFeatureId,
    EmptyFeatureUnits,
    EmptyChannelSemantics,
    EmptyFeatureDefinitionReference,
    EmptySourceReference,
    InvalidSourceDigest,
    InvalidInterval,
    ReferenceMustEndBeforeEvaluation,
    InvalidMonthWindow,
    InvalidMinimumReferenceYears,
    InvalidScaleFloor,
    InvalidAlertThreshold,
    EmptyAlertThresholdPolicyReference,
    InvalidCusumDriftAllowance,
    InvalidCusumAlertThreshold,
    InvalidCusumMinimumObservations,
    EmptyObservations,
    ObservationMonthInvalid { month_ordinal: i64 },
    ObservationsNotStrictlyIncreasing,
    InvalidObservationValue { month_ordinal: i64 },
    ArithmeticOverflow { month_ordinal: i64 },
}

impl fmt::Display for SeasonalSrdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage seasonal SRD error: {self:?}")
    }
}

impl std::error::Error for SeasonalSrdError {}

/// Convert Gregorian year/month to the module's integer month ordinal.
pub fn seasonal_month_ordinal(year: i32, month: u8) -> Option<i64> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    i64::from(year)
        .checked_mul(12)?
        .checked_add(i64::from(month) - 1)
}

fn year_month_from_ordinal(ordinal: i64) -> Option<(i32, u8)> {
    let year = ordinal.div_euclid(12);
    let month = ordinal.rem_euclid(12) + 1;
    if !(1..=9999).contains(&year) {
        return None;
    }
    Some((i32::try_from(year).ok()?, u8::try_from(month).ok()?))
}

fn valid_interval(interval: SeasonalInterval) -> bool {
    interval.start_month <= interval.end_month
        && year_month_from_ordinal(interval.start_month).is_some()
        && year_month_from_ordinal(interval.end_month).is_some()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn month_of_year(ordinal: i64) -> u8 {
    ordinal.rem_euclid(12) as u8
}

fn circular_month_distance(left: u8, right: u8) -> u8 {
    let direct = left.abs_diff(right);
    direct.min(12 - direct)
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let middle = ordered.len() / 2;
    if ordered.len() % 2 == 1 {
        Some(ordered[middle])
    } else {
        // Halve before adding to avoid overflow when both middle values are large.
        Some(ordered[middle - 1] / 2.0 + ordered[middle] / 2.0)
    }
}

fn median_absolute_deviation(values: &[f64], center: f64) -> Option<f64> {
    let deviations = values
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    if deviations.iter().any(|value| !value.is_finite()) {
        return None;
    }
    median(&deviations)
}

fn validate(
    series: &SeasonalSrdSeries,
    config: &SeasonalSrdConfig,
) -> Result<(), SeasonalSrdError> {
    let metadata = &series.metadata;
    if metadata.series_id.trim().is_empty() {
        return Err(SeasonalSrdError::EmptySeriesId);
    }
    if metadata.site_id.trim().is_empty() {
        return Err(SeasonalSrdError::EmptySiteId);
    }
    if metadata.feature_id.trim().is_empty() {
        return Err(SeasonalSrdError::EmptyFeatureId);
    }
    if metadata.feature_units.trim().is_empty() {
        return Err(SeasonalSrdError::EmptyFeatureUnits);
    }
    if metadata.channel_semantics.trim().is_empty() {
        return Err(SeasonalSrdError::EmptyChannelSemantics);
    }
    if metadata.feature_definition_ref.trim().is_empty() {
        return Err(SeasonalSrdError::EmptyFeatureDefinitionReference);
    }
    if metadata.source_ref.trim().is_empty() {
        return Err(SeasonalSrdError::EmptySourceReference);
    }
    if !valid_sha256(&metadata.source_asset_sha256) {
        return Err(SeasonalSrdError::InvalidSourceDigest);
    }
    if !valid_interval(config.reference_interval) || !valid_interval(config.evaluation_interval) {
        return Err(SeasonalSrdError::InvalidInterval);
    }
    if config.reference_interval.end_month >= config.evaluation_interval.start_month {
        return Err(SeasonalSrdError::ReferenceMustEndBeforeEvaluation);
    }
    if config.month_window > 6 {
        return Err(SeasonalSrdError::InvalidMonthWindow);
    }
    if config.min_reference_years == 0 {
        return Err(SeasonalSrdError::InvalidMinimumReferenceYears);
    }
    if !config.scale_floor.is_finite() || config.scale_floor <= 0.0 {
        return Err(SeasonalSrdError::InvalidScaleFloor);
    }
    if !config.alert_threshold_abs.is_finite() || config.alert_threshold_abs <= 0.0 {
        return Err(SeasonalSrdError::InvalidAlertThreshold);
    }
    if config.alert_threshold_policy_ref.trim().is_empty() {
        return Err(SeasonalSrdError::EmptyAlertThresholdPolicyReference);
    }
    if let Some(cusum) = config.cusum {
        if !cusum.drift_allowance.is_finite() || cusum.drift_allowance < 0.0 {
            return Err(SeasonalSrdError::InvalidCusumDriftAllowance);
        }
        if !cusum.alert_threshold.is_finite() || cusum.alert_threshold <= 0.0 {
            return Err(SeasonalSrdError::InvalidCusumAlertThreshold);
        }
        if cusum.min_valid_observations == 0 {
            return Err(SeasonalSrdError::InvalidCusumMinimumObservations);
        }
    }
    if series.observations.is_empty() {
        return Err(SeasonalSrdError::EmptyObservations);
    }

    let mut previous = None;
    for observation in &series.observations {
        if year_month_from_ordinal(observation.month_ordinal).is_none() {
            return Err(SeasonalSrdError::ObservationMonthInvalid {
                month_ordinal: observation.month_ordinal,
            });
        }
        if previous.is_some_and(|month| observation.month_ordinal <= month) {
            return Err(SeasonalSrdError::ObservationsNotStrictlyIncreasing);
        }
        if observation.value.is_some_and(|value| !value.is_finite()) {
            return Err(SeasonalSrdError::InvalidObservationValue {
                month_ordinal: observation.month_ordinal,
            });
        }
        previous = Some(observation.month_ordinal);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ReferenceStats {
    observation_count: usize,
    distinct_year_count: usize,
    median: Option<f64>,
    mad: Option<f64>,
    robust_scale: Option<f64>,
}

fn reference_stats(
    observations: &[SeasonalObservation],
    month: i64,
    config: &SeasonalSrdConfig,
) -> Result<ReferenceStats, SeasonalSrdError> {
    let target_month = month_of_year(month);
    let mut values = Vec::new();
    let mut years = BTreeSet::new();

    for observation in observations {
        if !(config.reference_interval.start_month..=config.reference_interval.end_month)
            .contains(&observation.month_ordinal)
        {
            continue;
        }
        let Some(value) = observation.value else {
            continue;
        };
        if circular_month_distance(target_month, month_of_year(observation.month_ordinal))
            <= config.month_window
        {
            values.push(value);
            let (year, _) = year_month_from_ordinal(observation.month_ordinal)
                .expect("observation months were validated");
            years.insert(year);
        }
    }

    let observation_count = values.len();
    let distinct_year_count = years.len();
    if distinct_year_count < config.min_reference_years {
        return Ok(ReferenceStats {
            observation_count,
            distinct_year_count,
            median: None,
            mad: None,
            robust_scale: None,
        });
    }

    let center = median(&values).ok_or(SeasonalSrdError::ArithmeticOverflow {
        month_ordinal: month,
    })?;
    let mad =
        median_absolute_deviation(&values, center).ok_or(SeasonalSrdError::ArithmeticOverflow {
            month_ordinal: month,
        })?;
    let robust_scale = (1.4826 * mad).max(config.scale_floor);
    if !center.is_finite() || !mad.is_finite() || !robust_scale.is_finite() {
        return Err(SeasonalSrdError::ArithmeticOverflow {
            month_ordinal: month,
        });
    }

    Ok(ReferenceStats {
        observation_count,
        distinct_year_count,
        median: Some(center),
        mad: Some(mad),
        robust_scale: Some(robust_scale),
    })
}

/// Evaluate a fixed-reference, month-of-year matched robust deviation series.
///
/// Reference statistics are frozen to reference_interval; all observations
/// at or after evaluation_interval.start_month are excluded from the
/// reference regardless of their values or availability. Missing evaluation
/// months are materialized as Unobserved outputs and never converted to 0.
pub fn evaluate_seasonal_srd(
    series: &SeasonalSrdSeries,
    config: &SeasonalSrdConfig,
) -> Result<SeasonalSrdReport, SeasonalSrdError> {
    validate(series, config)?;

    let observations_by_month = series
        .observations
        .iter()
        .map(|observation| (observation.month_ordinal, observation.value))
        .collect::<BTreeMap<_, _>>();

    let mut points = Vec::new();
    let mut scored_month_count = 0usize;
    let mut unobserved_month_count = 0usize;
    let mut insufficient_reference_month_count = 0usize;
    let mut single_month_alert_count = 0usize;
    let mut cusum_alert_count = 0usize;

    let mut cusum_positive = 0.0;
    let mut cusum_negative = 0.0;
    let mut cusum_valid_observation_count = 0usize;
    let mut previous_scored_month: Option<i64> = None;

    let mut month = config.evaluation_interval.start_month;
    loop {
        let stats = reference_stats(&series.observations, month, config)?;
        let observation_value = observations_by_month.get(&month).copied().flatten();

        let mut point = SeasonalSrdPoint {
            month_ordinal: month,
            state: SeasonalScoreState::Unobserved,
            observation_value,
            reference_observation_count: stats.observation_count,
            reference_year_count: stats.distinct_year_count,
            reference_median: stats.median,
            reference_mad: stats.mad,
            robust_scale: stats.robust_scale,
            signed_residual: None,
            robust_z_score: None,
            single_month_alert: None,
            cusum_positive: None,
            cusum_negative: None,
            cusum_valid_observation_count: None,
            cusum_reset_applied: None,
            cusum_alert: None,
        };

        if let Some(value) = observation_value {
            if stats.distinct_year_count < config.min_reference_years {
                point.state = SeasonalScoreState::InsufficientReference {
                    observed_reference_years: stats.distinct_year_count,
                    required_reference_years: config.min_reference_years,
                };
                insufficient_reference_month_count += 1;
            } else {
                let center = stats.median.expect("sufficient reference has median");
                let scale = stats
                    .robust_scale
                    .expect("sufficient reference has robust scale");
                let residual = value - center;
                let robust_z = residual / scale;
                if !residual.is_finite() || !robust_z.is_finite() {
                    return Err(SeasonalSrdError::ArithmeticOverflow {
                        month_ordinal: month,
                    });
                }
                let alert = robust_z.abs() >= config.alert_threshold_abs;
                point.state = SeasonalScoreState::Scored;
                point.signed_residual = Some(residual);
                point.robust_z_score = Some(robust_z);
                point.single_month_alert = Some(alert);
                scored_month_count += 1;
                if alert {
                    single_month_alert_count += 1;
                }

                if let Some(cusum) = config.cusum {
                    let mut reset_applied = false;
                    if let Some(previous_month) = previous_scored_month {
                        let missing_months = month
                            .checked_sub(previous_month)
                            .and_then(|delta| delta.checked_sub(1))
                            .ok_or(SeasonalSrdError::ArithmeticOverflow {
                                month_ordinal: month,
                            })?;
                        if missing_months > i64::from(cusum.reset_after_missing_months) {
                            cusum_positive = 0.0;
                            cusum_negative = 0.0;
                            cusum_valid_observation_count = 0;
                            reset_applied = true;
                        }
                    } else {
                        reset_applied = true;
                    }

                    cusum_positive = (cusum_positive + robust_z - cusum.drift_allowance).max(0.0);
                    cusum_negative = (cusum_negative - robust_z - cusum.drift_allowance).max(0.0);
                    if !cusum_positive.is_finite() || !cusum_negative.is_finite() {
                        return Err(SeasonalSrdError::ArithmeticOverflow {
                            month_ordinal: month,
                        });
                    }
                    cusum_valid_observation_count = cusum_valid_observation_count.saturating_add(1);
                    let cusum_alert = cusum_valid_observation_count >= cusum.min_valid_observations
                        && (cusum_positive >= cusum.alert_threshold
                            || cusum_negative >= cusum.alert_threshold);
                    point.cusum_positive = Some(cusum_positive);
                    point.cusum_negative = Some(cusum_negative);
                    point.cusum_valid_observation_count = Some(cusum_valid_observation_count);
                    point.cusum_reset_applied = Some(reset_applied);
                    point.cusum_alert = Some(cusum_alert);
                    if cusum_alert {
                        cusum_alert_count += 1;
                    }
                    previous_scored_month = Some(month);
                }
            }
        } else {
            unobserved_month_count += 1;
        }

        points.push(point);
        if month == config.evaluation_interval.end_month {
            break;
        }
        month = month
            .checked_add(1)
            .ok_or(SeasonalSrdError::ArithmeticOverflow {
                month_ordinal: month,
            })?;
    }

    Ok(SeasonalSrdReport {
        schema_version: SEASONAL_SRD_SCHEMA_VERSION.to_owned(),
        metadata: series.metadata.clone(),
        config: config.clone(),
        scored_month_count,
        unobserved_month_count,
        insufficient_reference_month_count,
        single_month_alert_count,
        cusum_alert_count,
        points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(year: i32, month: u8) -> i64 {
        seasonal_month_ordinal(year, month).unwrap()
    }

    fn metadata() -> SeasonalFeatureMetadata {
        SeasonalFeatureMetadata {
            series_id: "synthetic-series".into(),
            site_id: "synthetic-site".into(),
            feature_id: "ndvi".into(),
            feature_units: "unitless".into(),
            channel_semantics: "Sentinel-2 B08 NIR and B04 red surface reflectance".into(),
            feature_definition_ref: "urn:synthetic:feature-definition:ndvi".into(),
            source_ref: "urn:synthetic:source:sentinel2-time-series".into(),
            source_asset_sha256: "a".repeat(64),
        }
    }

    fn config() -> SeasonalSrdConfig {
        SeasonalSrdConfig {
            reference_interval: SeasonalInterval {
                start_month: month(2020, 1),
                end_month: month(2022, 12),
            },
            evaluation_interval: SeasonalInterval {
                start_month: month(2023, 1),
                end_month: month(2023, 3),
            },
            month_window: 0,
            min_reference_years: 3,
            scale_floor: 1.0,
            alert_threshold_abs: 3.0,
            alert_threshold_policy_ref: "urn:synthetic:threshold-policy:v1".into(),
            cusum: None,
        }
    }

    fn observations_for_months(
        values_by_month: &[(u8, f64)],
        evaluation: &[(i32, u8, Option<f64>)],
    ) -> Vec<SeasonalObservation> {
        let mut values = Vec::new();
        for year in 2020..=2022 {
            for (month_number, value) in values_by_month {
                values.push(SeasonalObservation {
                    month_ordinal: month(year, *month_number),
                    value: Some(*value),
                });
            }
        }
        for (year, month_number, value) in evaluation {
            values.push(SeasonalObservation {
                month_ordinal: month(*year, *month_number),
                value: *value,
            });
        }
        values.sort_by_key(|observation| observation.month_ordinal);
        values
    }

    fn series(observations: Vec<SeasonalObservation>) -> SeasonalSrdSeries {
        SeasonalSrdSeries {
            metadata: metadata(),
            observations,
        }
    }

    #[test]
    fn deterministic_seasonality_is_not_alerted_when_reference_is_sufficient() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 20.0), (3, 30.0)],
            &[
                (2023, 1, Some(10.0)),
                (2023, 2, Some(20.0)),
                (2023, 3, Some(30.0)),
            ],
        ));
        let report = evaluate_seasonal_srd(&input, &config()).unwrap();
        assert_eq!(report.scored_month_count, 3);
        assert_eq!(report.single_month_alert_count, 0);
        assert!(
            report
                .points
                .iter()
                .all(|point| point.robust_z_score == Some(0.0))
        );
    }

    #[test]
    fn injected_shift_produces_signed_robust_deviation_not_probability() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 20.0), (3, 30.0)],
            &[
                (2023, 1, Some(19.0)),
                (2023, 2, Some(20.0)),
                (2023, 3, Some(30.0)),
            ],
        ));
        let report = evaluate_seasonal_srd(&input, &config()).unwrap();
        assert_eq!(report.points[0].robust_z_score, Some(9.0));
        assert_eq!(report.points[0].single_month_alert, Some(true));
        assert_eq!(report.points[0].observation_value, Some(19.0));
    }

    #[test]
    fn insufficient_distinct_reference_years_abstains_instead_of_scoring_zero() {
        let input = series(vec![
            SeasonalObservation {
                month_ordinal: month(2021, 1),
                value: Some(10.0),
            },
            SeasonalObservation {
                month_ordinal: month(2022, 1),
                value: Some(10.0),
            },
            SeasonalObservation {
                month_ordinal: month(2023, 1),
                value: Some(10.0),
            },
        ]);
        let mut cfg = config();
        cfg.min_reference_years = 3;
        let report = evaluate_seasonal_srd(&input, &cfg).unwrap();
        assert!(matches!(
            report.points[0].state,
            SeasonalScoreState::InsufficientReference {
                observed_reference_years: 2,
                required_reference_years: 3
            }
        ));
        assert_eq!(report.points[0].robust_z_score, None);
        assert_eq!(report.insufficient_reference_month_count, 1);
    }

    #[test]
    fn missing_observation_is_unobserved_not_zero() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 20.0), (3, 30.0)],
            &[
                (2023, 1, None),
                (2023, 2, Some(20.0)),
                (2023, 3, Some(30.0)),
            ],
        ));
        let report = evaluate_seasonal_srd(&input, &config()).unwrap();
        assert_eq!(report.points[0].state, SeasonalScoreState::Unobserved);
        assert_eq!(report.points[0].observation_value, None);
        assert_eq!(report.points[0].single_month_alert, None);
    }

    #[test]
    fn reference_is_fixed_and_future_observations_cannot_leak_backwards() {
        let observations = observations_for_months(
            &[(1, 10.0), (2, 20.0), (3, 30.0)],
            &[
                (2023, 1, Some(10.0)),
                (2023, 2, Some(20.0)),
                (2023, 3, Some(30.0)),
                (2024, 1, Some(1000.0)),
            ],
        );
        let report = evaluate_seasonal_srd(&series(observations), &config()).unwrap();
        assert_eq!(report.points[0].reference_median, Some(10.0));
        assert_eq!(report.points[0].robust_z_score, Some(0.0));
    }

    #[test]
    fn circular_month_window_wraps_across_december_and_january() {
        let input = series(vec![
            SeasonalObservation {
                month_ordinal: month(2020, 12),
                value: Some(5.0),
            },
            SeasonalObservation {
                month_ordinal: month(2021, 1),
                value: Some(5.0),
            },
            SeasonalObservation {
                month_ordinal: month(2022, 2),
                value: Some(5.0),
            },
            SeasonalObservation {
                month_ordinal: month(2023, 1),
                value: Some(5.0),
            },
            SeasonalObservation {
                month_ordinal: month(2023, 2),
                value: Some(5.0),
            },
            SeasonalObservation {
                month_ordinal: month(2023, 3),
                value: Some(5.0),
            },
        ]);
        let mut cfg = config();
        cfg.month_window = 1;
        cfg.reference_interval = SeasonalInterval {
            start_month: month(2020, 1),
            end_month: month(2022, 12),
        };
        let report = evaluate_seasonal_srd(&input, &cfg).unwrap();
        assert_eq!(report.points[0].reference_year_count, 3);
        assert_eq!(report.points[0].robust_z_score, Some(0.0));
    }

    #[test]
    fn cusum_detects_a_persistent_shift_separately_from_single_month_alerts() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 10.0), (3, 10.0)],
            &[
                (2023, 1, Some(12.0)),
                (2023, 2, Some(12.0)),
                (2023, 3, Some(12.0)),
            ],
        ));
        let mut cfg = config();
        cfg.alert_threshold_abs = 3.0;
        cfg.cusum = Some(SeasonalCusumConfig {
            drift_allowance: 0.5,
            alert_threshold: 4.0,
            min_valid_observations: 2,
            reset_after_missing_months: 0,
        });
        let report = evaluate_seasonal_srd(&input, &cfg).unwrap();
        assert_eq!(report.single_month_alert_count, 0);
        assert_eq!(report.points[0].cusum_alert, Some(false));
        assert_eq!(report.points[1].cusum_alert, Some(false));
        assert_eq!(report.points[2].cusum_alert, Some(true));
        assert_eq!(report.cusum_alert_count, 1);
    }

    #[test]
    fn cusum_resets_after_missing_months_beyond_the_declared_gap() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 10.0), (3, 10.0)],
            &[(2023, 1, Some(12.0)), (2023, 3, Some(12.0))],
        ));
        let mut cfg = config();
        cfg.evaluation_interval = SeasonalInterval {
            start_month: month(2023, 1),
            end_month: month(2023, 3),
        };
        cfg.cusum = Some(SeasonalCusumConfig {
            drift_allowance: 0.5,
            alert_threshold: 3.0,
            min_valid_observations: 1,
            reset_after_missing_months: 0,
        });
        let report = evaluate_seasonal_srd(&input, &cfg).unwrap();
        assert_eq!(report.points[2].cusum_reset_applied, Some(true));
        assert_eq!(report.points[2].cusum_alert, Some(false));
    }

    #[test]
    fn invalid_values_month_order_metadata_and_reference_overlap_fail_closed() {
        let mut input = series(vec![
            SeasonalObservation {
                month_ordinal: month(2020, 1),
                value: Some(10.0),
            },
            SeasonalObservation {
                month_ordinal: month(2020, 1),
                value: Some(11.0),
            },
            SeasonalObservation {
                month_ordinal: month(2023, 1),
                value: Some(10.0),
            },
        ]);
        assert_eq!(
            evaluate_seasonal_srd(&input, &config()),
            Err(SeasonalSrdError::ObservationsNotStrictlyIncreasing)
        );

        input.observations[1].month_ordinal = month(2020, 2);
        input.observations[1].value = Some(f64::NAN);
        assert!(matches!(
            evaluate_seasonal_srd(&input, &config()),
            Err(SeasonalSrdError::InvalidObservationValue { .. })
        ));

        input.observations[1].value = Some(10.0);
        input.metadata.channel_semantics.clear();
        assert_eq!(
            evaluate_seasonal_srd(&input, &config()),
            Err(SeasonalSrdError::EmptyChannelSemantics)
        );

        input.metadata.channel_semantics = "verified feature".into();
        let mut overlapping = config();
        overlapping.reference_interval.end_month = month(2023, 1);
        assert_eq!(
            evaluate_seasonal_srd(&input, &overlapping),
            Err(SeasonalSrdError::ReferenceMustEndBeforeEvaluation)
        );
    }

    #[test]
    fn threshold_provenance_and_cusum_configuration_are_required() {
        let input = series(observations_for_months(
            &[(1, 10.0), (2, 20.0), (3, 30.0)],
            &[
                (2023, 1, Some(10.0)),
                (2023, 2, Some(20.0)),
                (2023, 3, Some(30.0)),
            ],
        ));
        let mut cfg = config();
        cfg.alert_threshold_policy_ref.clear();
        assert_eq!(
            evaluate_seasonal_srd(&input, &cfg),
            Err(SeasonalSrdError::EmptyAlertThresholdPolicyReference)
        );

        let mut cfg = config();
        cfg.cusum = Some(SeasonalCusumConfig {
            drift_allowance: f64::NAN,
            alert_threshold: 4.0,
            min_valid_observations: 2,
            reset_after_missing_months: 1,
        });
        assert_eq!(
            evaluate_seasonal_srd(&input, &cfg),
            Err(SeasonalSrdError::InvalidCusumDriftAllowance)
        );
    }
}
