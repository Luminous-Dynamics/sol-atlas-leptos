// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fixed-reference seasonal robust-deviation baseline.
//! Missing or insufficient observations abstain; anomaly strength is not probability.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const SEASONAL_BASELINE_SCHEMA_VERSION: &str = "luminous.heritage-seasonal-baseline.v0.1";
pub const MAX_EVALUATION_MONTHS: u64 = 240;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureObservation {
    pub month_ordinal: i64,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalFeatureSeries {
    pub feature_id: String,
    pub units: String,
    pub source_ref: String,
    pub feature_definition_sha256: String,
    pub observations: Vec<FeatureObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeasonalBaselineConfig {
    pub reference_start_month: i64,
    pub reference_end_month: i64,
    pub evaluation_start_month: i64,
    pub evaluation_end_month: i64,
    pub minimum_reference_years: usize,
    pub scale_floor: f64,
    pub robust_z_alert_threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonalScoreStatus {
    Scored,
    MissingObservation,
    InsufficientReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalDeviation {
    pub month_ordinal: i64,
    pub status: SeasonalScoreStatus,
    pub reference_sample_count: usize,
    pub reference_year_count: usize,
    pub reference_median: Option<f64>,
    pub reference_mad: Option<f64>,
    pub signed_robust_z: Option<f64>,
    pub absolute_robust_z: Option<f64>,
    pub anomaly_strength: Option<f64>,
    pub alert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeasonalBaselineReport {
    pub schema_version: String,
    pub feature_id: String,
    pub units: String,
    pub source_ref: String,
    pub feature_definition_sha256: String,
    pub reference_start_month: i64,
    pub reference_end_month: i64,
    pub evaluation_start_month: i64,
    pub evaluation_end_month: i64,
    pub minimum_reference_years: usize,
    pub scale_floor: f64,
    pub expected_evaluation_months: u64,
    pub supplied_evaluation_months: usize,
    pub unobserved_evaluation_months: u64,
    pub unobserved_month_ordinals: Vec<i64>,
    pub valid_evaluation_observations: usize,
    /// Explicitly present observations whose value is None.
    pub missing_evaluation_observations: usize,
    pub insufficient_reference_observations: usize,
    pub scored_observations: usize,
    pub unscored_evaluation_months: u64,
    /// Non-null observations divided by expected evaluation months.
    pub observation_coverage: f64,
    /// Months with an actual robust score divided by expected evaluation months.
    pub score_coverage: f64,
    pub alert_count: usize,
    pub robust_z_alert_threshold: f64,
    pub results: Vec<SeasonalDeviation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeasonalBaselineError {
    EmptyFeatureId,
    EmptyUnits,
    EmptySourceReference,
    InvalidFeatureDefinitionDigest,
    InvalidMonthOrdinal { month_ordinal: i64 },
    InvalidReferenceWindow,
    InvalidEvaluationWindow,
    EvaluationWindowTooLong,
    ReferenceOverlapsEvaluation,
    InvalidMinimumReferenceYears,
    InvalidScaleFloor,
    InvalidAlertThreshold,
    NonFiniteRobustStatistics { month_ordinal: i64 },
    NonFiniteRobustScore { month_ordinal: i64 },
    DuplicateMonth { month_ordinal: i64 },
    NonFiniteFeatureValue { month_ordinal: i64 },
    EmptyObservationSeries,
}

impl fmt::Display for SeasonalBaselineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage seasonal baseline error: {self:?}")
    }
}

impl std::error::Error for SeasonalBaselineError {}

fn valid_month_ordinal(value: i64) -> bool {
    (1..=9999).contains(&value.div_euclid(12))
}

fn month_of_year(value: i64) -> i64 {
    value.rem_euclid(12)
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[middle])
    } else {
        Some(values[middle - 1] * 0.5 + values[middle] * 0.5)
    }
}

fn validate(
    series: &SeasonalFeatureSeries,
    config: SeasonalBaselineConfig,
) -> Result<(), SeasonalBaselineError> {
    if series.feature_id.trim().is_empty() {
        return Err(SeasonalBaselineError::EmptyFeatureId);
    }
    if series.units.trim().is_empty() {
        return Err(SeasonalBaselineError::EmptyUnits);
    }
    if series.source_ref.trim().is_empty() {
        return Err(SeasonalBaselineError::EmptySourceReference);
    }
    if !is_sha256(&series.feature_definition_sha256) {
        return Err(SeasonalBaselineError::InvalidFeatureDefinitionDigest);
    }
    if series.observations.is_empty() {
        return Err(SeasonalBaselineError::EmptyObservationSeries);
    }
    if !valid_month_ordinal(config.reference_start_month)
        || !valid_month_ordinal(config.reference_end_month)
        || config.reference_start_month > config.reference_end_month
    {
        return Err(SeasonalBaselineError::InvalidReferenceWindow);
    }
    if !valid_month_ordinal(config.evaluation_start_month)
        || !valid_month_ordinal(config.evaluation_end_month)
        || config.evaluation_start_month > config.evaluation_end_month
    {
        return Err(SeasonalBaselineError::InvalidEvaluationWindow);
    }
    let evaluation_months = u64::try_from(
        config.evaluation_end_month - config.evaluation_start_month + 1,
    ).map_err(|_| SeasonalBaselineError::InvalidEvaluationWindow)?;
    if evaluation_months > MAX_EVALUATION_MONTHS {
        return Err(SeasonalBaselineError::EvaluationWindowTooLong);
    }
    if config.reference_end_month >= config.evaluation_start_month {
        return Err(SeasonalBaselineError::ReferenceOverlapsEvaluation);
    }
    if config.minimum_reference_years < 2 {
        return Err(SeasonalBaselineError::InvalidMinimumReferenceYears);
    }
    if !config.scale_floor.is_finite() || config.scale_floor <= 0.0 {
        return Err(SeasonalBaselineError::InvalidScaleFloor);
    }
    if !config.robust_z_alert_threshold.is_finite()
        || config.robust_z_alert_threshold <= 0.0
    {
        return Err(SeasonalBaselineError::InvalidAlertThreshold);
    }

    let mut previous = None;
    for observation in &series.observations {
        if !valid_month_ordinal(observation.month_ordinal) {
            return Err(SeasonalBaselineError::InvalidMonthOrdinal {
                month_ordinal: observation.month_ordinal,
            });
        }
        if previous.is_some_and(|last| observation.month_ordinal <= last) {
            return Err(SeasonalBaselineError::DuplicateMonth {
                month_ordinal: observation.month_ordinal,
            });
        }
        if observation.value.is_some_and(|value| !value.is_finite()) {
            return Err(SeasonalBaselineError::NonFiniteFeatureValue {
                month_ordinal: observation.month_ordinal,
            });
        }
        previous = Some(observation.month_ordinal);
    }
    Ok(())
}

fn reference_for_month(
    observations: &[FeatureObservation],
    target_month: i64,
    config: SeasonalBaselineConfig,
) -> (Vec<f64>, usize) {
    let target_month_of_year = month_of_year(target_month);
    let mut values = Vec::new();
    let mut years = BTreeSet::new();

    for observation in observations {
        if observation.month_ordinal < config.reference_start_month
            || observation.month_ordinal > config.reference_end_month
            || month_of_year(observation.month_ordinal) != target_month_of_year
        {
            continue;
        }
        if let Some(value) = observation.value {
            values.push(value);
            years.insert(observation.month_ordinal.div_euclid(12));
        }
    }
    (values, years.len())
}

/// Score evaluation months against a fixed, earlier same-month-of-year reference.
///
/// For each evaluation month, the reference is the median and MAD of valid
/// values for that calendar month inside the preregistered reference window.
/// The robust z-score is signed; anomaly_strength is 1-exp(-abs(z)) and is not
/// a calibrated probability. No future values or missing months enter the
/// reference distribution.
pub fn evaluate_seasonal_robust_deviation(
    series: &SeasonalFeatureSeries,
    config: SeasonalBaselineConfig,
) -> Result<SeasonalBaselineReport, SeasonalBaselineError> {
    validate(series, config)?;

    let expected_months = u64::try_from(
        config.evaluation_end_month - config.evaluation_start_month + 1,
    )
    .expect("validated month span fits u64");

    let evaluation: Vec<&FeatureObservation> = series.observations.iter()
        .filter(|observation| {
            observation.month_ordinal >= config.evaluation_start_month
                && observation.month_ordinal <= config.evaluation_end_month
        })
        .collect();
    let supplied_months: BTreeSet<i64> = evaluation
        .iter()
        .map(|observation| observation.month_ordinal)
        .collect();
    let unobserved_month_ordinals: Vec<i64> =
        (config.evaluation_start_month..=config.evaluation_end_month)
            .filter(|month| !supplied_months.contains(month))
            .collect();

    let mut results = Vec::with_capacity(evaluation.len());
    let mut valid_count = 0;
    let mut missing_count = 0;
    let mut insufficient_count = 0;
    let mut scored_count = 0;
    let mut alert_count = 0;

    for observation in evaluation {
        let Some(value) = observation.value else {
            missing_count += 1;
            results.push(SeasonalDeviation {
                month_ordinal: observation.month_ordinal,
                status: SeasonalScoreStatus::MissingObservation,
                reference_sample_count: 0,
                reference_year_count: 0,
                reference_median: None,
                reference_mad: None,
                signed_robust_z: None,
                absolute_robust_z: None,
                anomaly_strength: None,
                alert: false,
            });
            continue;
        };
        valid_count += 1;

        let (mut reference_values, reference_year_count) =
            reference_for_month(&series.observations, observation.month_ordinal, config);

        if reference_year_count < config.minimum_reference_years {
            insufficient_count += 1;
            results.push(SeasonalDeviation {
                month_ordinal: observation.month_ordinal,
                status: SeasonalScoreStatus::InsufficientReference,
                reference_sample_count: reference_values.len(),
                reference_year_count,
                reference_median: None,
                reference_mad: None,
                signed_robust_z: None,
                absolute_robust_z: None,
                anomaly_strength: None,
                alert: false,
            });
            continue;
        }

        let reference_median =
            median(&mut reference_values).expect("valid reference years imply samples");
        let mut deviations: Vec<f64> = reference_values.iter()
            .map(|sample| (sample - reference_median).abs())
            .collect();
        if !reference_median.is_finite() || deviations.iter().any(|value| !value.is_finite()) {
            return Err(SeasonalBaselineError::NonFiniteRobustStatistics {
                month_ordinal: observation.month_ordinal,
            });
        }
        let reference_mad = median(&mut deviations).expect("reference values are non-empty");
        let raw_scale = 1.4826 * reference_mad;
        if !reference_mad.is_finite() || !raw_scale.is_finite() {
            return Err(SeasonalBaselineError::NonFiniteRobustStatistics {
                month_ordinal: observation.month_ordinal,
            });
        }
        let scale = raw_scale.max(config.scale_floor);
        let signed_z = (value - reference_median) / scale;
        if !signed_z.is_finite() {
            return Err(SeasonalBaselineError::NonFiniteRobustScore {
                month_ordinal: observation.month_ordinal,
            });
        }
        let absolute_z = signed_z.abs();
        let strength = 1.0 - (-absolute_z).exp();
        let alert = absolute_z >= config.robust_z_alert_threshold;

        scored_count += 1;
        if alert {
            alert_count += 1;
        }

        results.push(SeasonalDeviation {
            month_ordinal: observation.month_ordinal,
            status: SeasonalScoreStatus::Scored,
            reference_sample_count: reference_values.len(),
            reference_year_count,
            reference_median: Some(reference_median),
            reference_mad: Some(reference_mad),
            signed_robust_z: Some(signed_z),
            absolute_robust_z: Some(absolute_z),
            anomaly_strength: Some(strength),
            alert,
        });
    }

    Ok(SeasonalBaselineReport {
        schema_version: SEASONAL_BASELINE_SCHEMA_VERSION.to_owned(),
        feature_id: series.feature_id.clone(),
        units: series.units.clone(),
        source_ref: series.source_ref.clone(),
        feature_definition_sha256: series.feature_definition_sha256.clone(),
        reference_start_month: config.reference_start_month,
        reference_end_month: config.reference_end_month,
        evaluation_start_month: config.evaluation_start_month,
        evaluation_end_month: config.evaluation_end_month,
        minimum_reference_years: config.minimum_reference_years,
        scale_floor: config.scale_floor,
        expected_evaluation_months: expected_months,
        supplied_evaluation_months: results.len(),
        unobserved_evaluation_months: unobserved_month_ordinals.len() as u64,
        unobserved_month_ordinals,
        valid_evaluation_observations: valid_count,
        missing_evaluation_observations: missing_count,
        insufficient_reference_observations: insufficient_count,
        scored_observations: scored_count,
        unscored_evaluation_months: expected_months.saturating_sub(scored_count as u64),
        observation_coverage: (valid_count as f64) / expected_months as f64,
        score_coverage: scored_count as f64 / expected_months as f64,
        alert_count,
        robust_z_alert_threshold: config.robust_z_alert_threshold,
        results,
    })
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn month(year: i64, month: i64) -> i64 {
        year * 12 + month - 1
    }

    fn config() -> SeasonalBaselineConfig {
        SeasonalBaselineConfig {
            reference_start_month: month(2021, 1),
            reference_end_month: month(2022, 12),
            evaluation_start_month: month(2023, 1),
            evaluation_end_month: month(2023, 12),
            minimum_reference_years: 2,
            scale_floor: 0.5,
            robust_z_alert_threshold: 3.5,
        }
    }

    fn series(observations: Vec<FeatureObservation>) -> SeasonalFeatureSeries {
        SeasonalFeatureSeries {
            feature_id: "synthetic-index".into(),
            units: "synthetic-units".into(),
            source_ref: "urn:synthetic:feature-series:01".into(),
            feature_definition_sha256: "a".repeat(64),
            observations,
        }
    }

    #[test]
    fn deterministic_seasonality_produces_no_false_alerts() {
        let mut observations = Vec::new();
        for year in 2021..=2022 {
            for month_value in 1..=12 {
                observations.push(FeatureObservation {
                    month_ordinal: month(year, month_value),
                    value: Some(10.0 + month_value as f64),
                });
            }
        }
        for month_value in 1..=12 {
            observations.push(FeatureObservation {
                month_ordinal: month(2023, month_value),
                value: Some(10.0 + month_value as f64),
            });
        }
        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();
        assert_eq!(report.scored_observations, 12);
        assert_eq!(report.alert_count, 0);
        assert!(report.results.iter().all(|item| item.signed_robust_z == Some(0.0)));
    }

    #[test]
    fn persistent_shift_produces_signed_deviations_and_alerts() {
        let mut observations = Vec::new();
        for year in 2021..=2022 {
            for month_value in 1..=12 {
                observations.push(FeatureObservation {
                    month_ordinal: month(year, month_value),
                    value: Some(20.0 + month_value as f64),
                });
            }
        }
        for month_value in 1..=12 {
            let shift = if month_value >= 6 { 8.0 } else { 0.0 };
            observations.push(FeatureObservation {
                month_ordinal: month(2023, month_value),
                value: Some(20.0 + month_value as f64 + shift),
            });
        }
        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();
        assert_eq!(report.alert_count, 7);
        assert!(report.results.iter().filter(|item| item.month_ordinal >= month(2023, 6))
            .all(|item| item.signed_robust_z.is_some_and(|z| z > 0.0 && item.alert)));
    }

    #[test]
    fn missing_observations_remain_unscored() {
        let mut observations = Vec::new();
        for year in 2021..=2022 {
            for month_value in 1..=12 {
                observations.push(FeatureObservation {
                    month_ordinal: month(year, month_value),
                    value: Some(month_value as f64),
                });
            }
        }
        observations.push(FeatureObservation {
            month_ordinal: month(2023, 1),
            value: None,
        });
        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();
        assert_eq!(report.missing_evaluation_observations, 1);
        assert_eq!(report.results[0].status, SeasonalScoreStatus::MissingObservation);
        assert_eq!(report.results[0].anomaly_strength, None);
    }

    #[test]
    fn insufficient_reference_years_causes_abstention() {
        let observations = vec![
            FeatureObservation { month_ordinal: month(2022, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2023, 1), value: Some(2.0) },
        ];
        let short_reference = SeasonalBaselineConfig {
            reference_start_month: month(2022, 1),
            reference_end_month: month(2022, 12),
            evaluation_start_month: month(2023, 1),
            evaluation_end_month: month(2023, 1),
            ..config()
        };
        let report = evaluate_seasonal_robust_deviation(&series(observations), short_reference).unwrap();
        assert_eq!(report.insufficient_reference_observations, 1);
        assert_eq!(report.results[0].status, SeasonalScoreStatus::InsufficientReference);
    }

    #[test]
    fn evaluation_window_is_bounded_and_report_captures_fixed_config() {
        let observations = vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2022, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2023, 1), value: Some(1.0) },
        ];
        let config = SeasonalBaselineConfig {
            evaluation_end_month: month(2043, 1),
            ..config()
        };
        assert_eq!(
            evaluate_seasonal_robust_deviation(&series(observations.clone()), config),
            Err(SeasonalBaselineError::EvaluationWindowTooLong)
        );

        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();
        assert_eq!(report.reference_start_month, month(2021, 1));
        assert_eq!(report.reference_end_month, month(2022, 12));
        assert_eq!(report.evaluation_start_month, month(2023, 1));
        assert_eq!(report.minimum_reference_years, 2);
        assert_eq!(report.scale_floor, 0.5);
    }

    #[test]
    fn extreme_finite_inputs_cannot_emit_infinite_scores_or_statistics() {
        let extreme = vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(-f64::MAX) },
            FeatureObservation { month_ordinal: month(2022, 1), value: Some(f64::MAX) },
            FeatureObservation { month_ordinal: month(2023, 1), value: Some(f64::MAX) },
        ];
        assert!(matches!(
            evaluate_seasonal_robust_deviation(&series(extreme), config()),
            Err(SeasonalBaselineError::NonFiniteRobustStatistics { .. })
                | Err(SeasonalBaselineError::NonFiniteRobustScore { .. })
        ));
    }

    #[test]
    fn absent_months_and_explicit_nulls_are_both_counted_as_unobserved() {
        let observations = vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2022, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2023, 1), value: Some(2.0) },
            FeatureObservation { month_ordinal: month(2023, 2), value: None },
        ];
        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();

        assert_eq!(report.expected_evaluation_months, 12);
        assert_eq!(report.supplied_evaluation_months, 2);
        assert_eq!(report.unobserved_evaluation_months, 10);
        assert_eq!(report.unobserved_month_ordinals, (month(2023, 2) + 1..=month(2023, 12)).collect::<Vec<_>>());
        assert_eq!(report.valid_evaluation_observations, 1);
        assert_eq!(report.missing_evaluation_observations, 1);
        assert_eq!(report.scored_observations, 1);
        assert_eq!(report.unscored_evaluation_months, 11);
        assert!((report.observation_coverage - (1.0 / 12.0)).abs() < 1e-12);
        assert!((report.score_coverage - (1.0 / 12.0)).abs() < 1e-12);
    }

    #[test]
    fn future_values_do_not_leak_into_reference_statistics() {
        let observations = vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2022, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2023, 1), value: Some(2.0) },
            FeatureObservation { month_ordinal: month(2024, 1), value: Some(1000.0) },
        ];
        let report = evaluate_seasonal_robust_deviation(&series(observations), config()).unwrap();
        assert_eq!(report.results[0].reference_median, Some(1.0));
        assert_eq!(report.results[0].signed_robust_z, Some(2.0));
    }

    #[test]
    fn invalid_windows_duplicate_months_and_non_finite_values_fail_closed() {
        let duplicate = series(vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(1.0) },
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(2.0) },
        ]);
        assert!(matches!(
            evaluate_seasonal_robust_deviation(&duplicate, config()),
            Err(SeasonalBaselineError::DuplicateMonth { .. })
        ));

        let non_finite = series(vec![
            FeatureObservation { month_ordinal: month(2021, 1), value: Some(f64::NAN) },
        ]);
        assert!(matches!(
            evaluate_seasonal_robust_deviation(&non_finite, config()),
            Err(SeasonalBaselineError::NonFiniteFeatureValue { .. })
        ));

        let overlap = SeasonalBaselineConfig {
            reference_end_month: month(2023, 1),
            ..config()
        };
        assert!(matches!(
            evaluate_seasonal_robust_deviation(&series(vec![
                FeatureObservation { month_ordinal: month(2021, 1), value: Some(1.0) },
            ]), overlap),
            Err(SeasonalBaselineError::ReferenceOverlapsEvaluation)
        ));
    }
}
