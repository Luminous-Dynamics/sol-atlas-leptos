//! Grid-facing demand-response metrics for evidence-bound compute scheduling.
//!
//! These helpers implement the published measurement definitions for regulation
//! tracking and emergency-demand-response curtailment; they do not simulate a
//! data center or prove that a baseline is a valid causal counterfactual.
//!
//! Reference: Acun et al., "Investigating Power Consumption Flexibility of AI
//! Data Centers for Demand Response Participation", ACM E-Energy 2026,
//! DOI: 10.1145/3744255.3798112. FlexDC-Sim's public reference source is pinned
//! separately in the benchmark cross-validation issue and must not be treated
//! as part of this Rust implementation.

use std::fmt;

/// Default normalized tracking-error threshold in the FlexDC-Sim paper's RSR
/// experiment: absolute(measured - target) / reserve must be no greater than 0.3.
pub const RSR_NORMALIZED_ERROR_THRESHOLD: f64 = 0.3;

/// The paper's empirical requirement is a strict upper bound:
/// the fraction of observations above the error threshold must be < 10%.
pub const RSR_MAX_ERROR_EXCEEDANCE_FRACTION: f64 = 0.10;

/// Percentile reported by FlexDC-Sim for normalized regulation tracking error.
pub const RSR_TRACKING_ERROR_QUANTILE: f64 = 0.90;

/// Typed errors: unavailable observations are rejected rather than silently
/// filtered, because dropping them can improve a tracking score artificially.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetricError {
    EmptySeries,
    LengthMismatch {
        series: &'static str,
        expected: usize,
        actual: usize,
    },
    MissingObservation {
        series: &'static str,
        index: usize,
    },
    InvalidObservation {
        series: &'static str,
        index: usize,
        reason: &'static str,
    },
    InvalidScalar {
        field: &'static str,
        reason: &'static str,
    },
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySeries => write!(f, "metric input series is empty"),
            Self::LengthMismatch {
                series,
                expected,
                actual,
            } => write!(
                f,
                "{series} length mismatch: expected {expected}, got {actual}"
            ),
            Self::MissingObservation { series, index } => {
                write!(f, "{series}[{index}] is unknown or missing")
            }
            Self::InvalidObservation {
                series,
                index,
                reason,
            } => write!(f, "{series}[{index}] is invalid: {reason}"),
            Self::InvalidScalar { field, reason } => {
                write!(f, "{field} is invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for MetricError {}

/// Regulation service reserve (RSR) tracking metrics over an aligned trace.
///
/// All power values are kW. target_power_kw and measured_power_kw are
/// interval-by-interval samples at the same time boundaries. Reserve must be
/// strictly positive; unlike a tiny epsilon in a denominator, missing/zero
/// reserve is rejected as insufficient evidence.
///
/// p90_normalized_absolute_error uses linear interpolation at rank q * (n - 1),
/// matching the default quantile behavior used by pandas in FlexDC-Sim's
/// extract_power_metrics.py. FlexDC-Sim adds 0.0001 to reserve in its runtime
/// denominator; this helper uses positive reserve exactly and rejects zero so
/// the small epsilon cannot mask an invalid reserve declaration. Exact parity
/// comparisons must record that difference.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct RegulationTrackingAssessment {
    pub sample_count: usize,
    pub reserve_capacity_kw: f64,
    pub normalized_absolute_errors: Vec<f64>,
    pub mean_normalized_absolute_error: f64,
    pub p90_normalized_absolute_error: f64,
    pub max_normalized_absolute_error: f64,
    /// FlexDC-Sim source-parity criterion: interpolated p90 absolute error <= threshold.
    pub p90_threshold_satisfied: bool,
    pub error_threshold: f64,
    pub max_allowed_exceedance_fraction: f64,
    pub threshold_exceedance_count: usize,
    pub threshold_exceedance_fraction: f64,
    /// Implements the strict inequality used in the reference paper:
    /// exceedance fraction < allowed fraction, not <=.
    pub exceedance_rate_requirement_satisfied: bool,
}

/// Tracking metrics computed from an already normalized source error series.
///
/// Use this form when reproducing a reference tool's exported normalized-error
/// column (for example FlexDC-Sim's `trackingError`). The signed values are
/// converted to absolute errors for quantiles and threshold checks. The source
/// tool's time alignment and denominator convention remain part of the input
/// provenance; this function does not validate either.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct NormalizedErrorTrackingAssessment {
    pub sample_count: usize,
    pub normalization_basis: &'static str,
    pub normalized_absolute_errors: Vec<f64>,
    pub mean_normalized_absolute_error: f64,
    pub p90_normalized_absolute_error: f64,
    pub max_normalized_absolute_error: f64,
    /// FlexDC-Sim source-parity criterion: interpolated p90 absolute error <= threshold.
    pub p90_threshold_satisfied: bool,
    pub error_threshold: f64,
    pub max_allowed_exceedance_fraction: f64,
    pub threshold_exceedance_count: usize,
    pub threshold_exceedance_fraction: f64,
    pub exceedance_rate_requirement_satisfied: bool,
}

/// Assess an already normalized signed tracking-error series.
///
/// This is deliberately separate from `assess_rsr_tracking`: it reproduces
/// metrics over the supplied normalized values without silently recalculating
/// the source's target alignment or reserve denominator.
pub fn assess_normalized_rsr_tracking_errors(
    signed_normalized_errors: &[Option<f64>],
    error_threshold: f64,
    max_allowed_exceedance_fraction: f64,
) -> Result<NormalizedErrorTrackingAssessment, MetricError> {
    if signed_normalized_errors.is_empty() {
        return Err(MetricError::EmptySeries);
    }
    require_nonnegative_fraction("error_threshold", error_threshold)?;
    require_unit_fraction(
        "max_allowed_exceedance_fraction",
        max_allowed_exceedance_fraction,
    )?;

    let mut errors = Vec::with_capacity(signed_normalized_errors.len());
    for (index, value) in signed_normalized_errors.iter().enumerate() {
        let value = value.ok_or(MetricError::MissingObservation {
            series: "signed_normalized_errors",
            index,
        })?;
        if !value.is_finite() {
            return Err(MetricError::InvalidObservation {
                series: "signed_normalized_errors",
                index,
                reason: "normalized error must be finite",
            });
        }
        let absolute_error = value.abs();
        if !absolute_error.is_finite() {
            return Err(MetricError::InvalidObservation {
                series: "normalized_absolute_errors",
                index,
                reason: "absolute normalized error is non-finite",
            });
        }
        errors.push(absolute_error);
    }

    let sample_count = errors.len();
    let mut mean_normalized_absolute_error = 0.0;
    for (index, error) in errors.iter().enumerate() {
        mean_normalized_absolute_error +=
            (*error - mean_normalized_absolute_error) / (index + 1) as f64;
        if !mean_normalized_absolute_error.is_finite() {
            return Err(MetricError::InvalidScalar {
                field: "mean_normalized_absolute_error",
                reason: "derived mean is non-finite",
            });
        }
    }
    let mut sorted_errors = errors.clone();
    sorted_errors.sort_by(f64::total_cmp);
    let p90_normalized_absolute_error =
        linear_quantile_sorted(&sorted_errors, RSR_TRACKING_ERROR_QUANTILE);
    let max_normalized_absolute_error = sorted_errors[sample_count - 1];
    let threshold_exceedance_count = errors
        .iter()
        .filter(|&&error| error > error_threshold)
        .count();
    let threshold_exceedance_fraction = threshold_exceedance_count as f64 / sample_count as f64;

    Ok(NormalizedErrorTrackingAssessment {
        sample_count,
        normalization_basis: "source_normalized_error_series",
        normalized_absolute_errors: errors,
        mean_normalized_absolute_error,
        p90_normalized_absolute_error,
        max_normalized_absolute_error,
        error_threshold,
        max_allowed_exceedance_fraction,
        threshold_exceedance_count,
        threshold_exceedance_fraction,
        p90_threshold_satisfied: p90_normalized_absolute_error <= error_threshold,
        exceedance_rate_requirement_satisfied: threshold_exceedance_fraction
            < max_allowed_exceedance_fraction,
    })
}

/// Assess RSR power tracking using the paper's default 0.3 error threshold and
/// strict <10% exceedance condition.
pub fn assess_reference_rsr_tracking(
    target_power_kw: &[Option<f64>],
    measured_power_kw: &[Option<f64>],
    reserve_capacity_kw: f64,
) -> Result<RegulationTrackingAssessment, MetricError> {
    assess_rsr_tracking(
        target_power_kw,
        measured_power_kw,
        reserve_capacity_kw,
        RSR_NORMALIZED_ERROR_THRESHOLD,
        RSR_MAX_ERROR_EXCEEDANCE_FRACTION,
    )
}

/// Assess normalized absolute power-target error over an aligned trace.
///
/// Error at sample i is absolute(measured_kw[i] - target_kw[i]) divided by
/// reserve_capacity_kw. No missing sample is skipped; one missing value
/// invalidates the assessment.
pub fn assess_rsr_tracking(
    target_power_kw: &[Option<f64>],
    measured_power_kw: &[Option<f64>],
    reserve_capacity_kw: f64,
    error_threshold: f64,
    max_allowed_exceedance_fraction: f64,
) -> Result<RegulationTrackingAssessment, MetricError> {
    if target_power_kw.is_empty() {
        return Err(MetricError::EmptySeries);
    }
    require_length(
        "measured_power_kw",
        target_power_kw.len(),
        measured_power_kw.len(),
    )?;
    require_positive_scalar("reserve_capacity_kw", reserve_capacity_kw)?;
    require_nonnegative_fraction("error_threshold", error_threshold)?;
    require_unit_fraction(
        "max_allowed_exceedance_fraction",
        max_allowed_exceedance_fraction,
    )?;

    let mut errors = Vec::with_capacity(target_power_kw.len());
    for (index, (target, measured)) in target_power_kw.iter().zip(measured_power_kw).enumerate() {
        let target = checked_power(*target, "target_power_kw", index)?;
        let measured = checked_power(*measured, "measured_power_kw", index)?;
        let normalized_error = (measured - target).abs() / reserve_capacity_kw;
        if !normalized_error.is_finite() {
            return Err(MetricError::InvalidObservation {
                series: "normalized_absolute_error",
                index,
                reason: "derived normalized error is non-finite",
            });
        }
        errors.push(normalized_error);
    }

    let sample_count = errors.len();
    let mut mean_normalized_absolute_error = 0.0;
    for (index, error) in errors.iter().enumerate() {
        mean_normalized_absolute_error +=
            (*error - mean_normalized_absolute_error) / (index + 1) as f64;
        if !mean_normalized_absolute_error.is_finite() {
            return Err(MetricError::InvalidScalar {
                field: "mean_normalized_absolute_error",
                reason: "derived mean is non-finite",
            });
        }
    }
    let mut sorted_errors = errors.clone();
    sorted_errors.sort_by(f64::total_cmp);
    let p90_normalized_absolute_error =
        linear_quantile_sorted(&sorted_errors, RSR_TRACKING_ERROR_QUANTILE);
    let max_normalized_absolute_error = sorted_errors[sample_count - 1];
    let threshold_exceedance_count = errors
        .iter()
        .filter(|&&error| error > error_threshold)
        .count();
    let threshold_exceedance_fraction = threshold_exceedance_count as f64 / sample_count as f64;

    Ok(RegulationTrackingAssessment {
        sample_count,
        reserve_capacity_kw,
        normalized_absolute_errors: errors,
        mean_normalized_absolute_error,
        p90_normalized_absolute_error,
        max_normalized_absolute_error,
        error_threshold,
        max_allowed_exceedance_fraction,
        threshold_exceedance_count,
        threshold_exceedance_fraction,
        p90_threshold_satisfied: p90_normalized_absolute_error <= error_threshold,
        exceedance_rate_requirement_satisfied: threshold_exceedance_fraction
            < max_allowed_exceedance_fraction,
    })
}

/// Emergency demand response (EDR) comparison against a supplied passive-load
/// or unconstrained baseline trace.
///
/// Powers are kW; durations are hours. The baseline must already have been
/// aligned to the same time intervals and workload assumptions. This function
/// computes accounting metrics only: it cannot establish the causal validity
/// of that baseline.
///
/// gross_curtailment_kwh counts positive reductions; rebound_energy_kwh counts
/// intervals where event power is above baseline. Net reduction may be negative
/// when rebound exceeds gross curtailment.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct CurtailmentAssessment {
    pub sample_count: usize,
    pub duration_hours: f64,
    pub baseline_energy_kwh: f64,
    pub event_energy_kwh: f64,
    pub net_energy_reduction_kwh: f64,
    pub net_energy_reduction_fraction: Option<f64>,
    pub gross_curtailment_kwh: f64,
    pub rebound_energy_kwh: f64,
    pub rebound_interval_count: usize,
    pub peak_baseline_power_kw: f64,
    pub peak_event_power_kw: f64,
    pub peak_event_power_excess_over_baseline_kw: f64,
}

/// Compare measured event power to an aligned declared baseline. Positive
/// curtailment and rebound are both reported to avoid hiding post-event peaks.
pub fn assess_edr_curtailment(
    baseline_power_kw: &[Option<f64>],
    event_power_kw: &[Option<f64>],
    interval_duration_hours: &[f64],
) -> Result<CurtailmentAssessment, MetricError> {
    if baseline_power_kw.is_empty() {
        return Err(MetricError::EmptySeries);
    }
    require_length(
        "event_power_kw",
        baseline_power_kw.len(),
        event_power_kw.len(),
    )?;
    require_length(
        "interval_duration_hours",
        baseline_power_kw.len(),
        interval_duration_hours.len(),
    )?;

    let mut baseline_energy_kwh = 0.0;
    let mut event_energy_kwh = 0.0;
    let mut gross_curtailment_kwh = 0.0;
    let mut rebound_energy_kwh = 0.0;
    let mut rebound_interval_count = 0_usize;
    let mut peak_baseline_power_kw = 0.0_f64;
    let mut peak_event_power_kw = 0.0_f64;
    let mut peak_event_power_excess_over_baseline_kw = 0.0_f64;
    let mut duration_hours = 0.0;

    for (index, ((baseline, event), duration)) in baseline_power_kw
        .iter()
        .zip(event_power_kw)
        .zip(interval_duration_hours)
        .enumerate()
    {
        let baseline = checked_power(*baseline, "baseline_power_kw", index)?;
        let event = checked_power(*event, "event_power_kw", index)?;
        if !duration.is_finite() || *duration <= 0.0 {
            return Err(MetricError::InvalidObservation {
                series: "interval_duration_hours",
                index,
                reason: "duration must be finite and strictly positive",
            });
        }

        let baseline_interval_kwh = baseline * *duration;
        let event_interval_kwh = event * *duration;
        let delta_energy_kwh = (baseline - event) * *duration;
        if !baseline_interval_kwh.is_finite()
            || !event_interval_kwh.is_finite()
            || !delta_energy_kwh.is_finite()
        {
            return Err(MetricError::InvalidObservation {
                series: "derived_energy_kwh",
                index,
                reason: "derived interval energy is non-finite",
            });
        }
        baseline_energy_kwh += baseline_interval_kwh;
        event_energy_kwh += event_interval_kwh;
        if delta_energy_kwh > 0.0 {
            gross_curtailment_kwh += delta_energy_kwh;
        } else if delta_energy_kwh < 0.0 {
            rebound_energy_kwh += -delta_energy_kwh;
            rebound_interval_count += 1;
        }
        duration_hours += *duration;
        if !baseline_energy_kwh.is_finite()
            || !event_energy_kwh.is_finite()
            || !gross_curtailment_kwh.is_finite()
            || !rebound_energy_kwh.is_finite()
            || !duration_hours.is_finite()
        {
            return Err(MetricError::InvalidObservation {
                series: "derived_energy_kwh",
                index,
                reason: "cumulative energy or duration is non-finite",
            });
        }
        peak_baseline_power_kw = peak_baseline_power_kw.max(baseline);
        peak_event_power_kw = peak_event_power_kw.max(event);
        peak_event_power_excess_over_baseline_kw =
            peak_event_power_excess_over_baseline_kw.max(event - baseline);
    }

    let net_energy_reduction_kwh = baseline_energy_kwh - event_energy_kwh;
    let net_energy_reduction_fraction =
        (baseline_energy_kwh > 0.0).then_some(net_energy_reduction_kwh / baseline_energy_kwh);
    if net_energy_reduction_fraction.is_some_and(|fraction| !fraction.is_finite()) {
        return Err(MetricError::InvalidScalar {
            field: "net_energy_reduction_fraction",
            reason: "derived reduction fraction is non-finite",
        });
    }

    Ok(CurtailmentAssessment {
        sample_count: baseline_power_kw.len(),
        duration_hours,
        baseline_energy_kwh,
        event_energy_kwh,
        net_energy_reduction_kwh,
        net_energy_reduction_fraction,
        gross_curtailment_kwh,
        rebound_energy_kwh,
        rebound_interval_count,
        peak_baseline_power_kw,
        peak_event_power_kw,
        peak_event_power_excess_over_baseline_kw,
    })
}

fn require_length(series: &'static str, expected: usize, actual: usize) -> Result<(), MetricError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MetricError::LengthMismatch {
            series,
            expected,
            actual,
        })
    }
}

fn require_positive_scalar(field: &'static str, value: f64) -> Result<(), MetricError> {
    if !value.is_finite() || value <= 0.0 {
        Err(MetricError::InvalidScalar {
            field,
            reason: "must be finite and strictly positive",
        })
    } else {
        Ok(())
    }
}

fn require_nonnegative_fraction(field: &'static str, value: f64) -> Result<(), MetricError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        Err(MetricError::InvalidScalar {
            field,
            reason: "must be finite and in [0, 1]",
        })
    } else {
        Ok(())
    }
}

fn require_unit_fraction(field: &'static str, value: f64) -> Result<(), MetricError> {
    require_nonnegative_fraction(field, value)
}

fn checked_power(
    value: Option<f64>,
    series: &'static str,
    index: usize,
) -> Result<f64, MetricError> {
    let value = value.ok_or(MetricError::MissingObservation { series, index })?;
    if !value.is_finite() {
        return Err(MetricError::InvalidObservation {
            series,
            index,
            reason: "power must be finite",
        });
    }
    if value < 0.0 {
        return Err(MetricError::InvalidObservation {
            series,
            index,
            reason: "power must be non-negative",
        });
    }
    Ok(value)
}

fn linear_quantile_sorted(sorted: &[f64], quantile: f64) -> f64 {
    debug_assert!(!sorted.is_empty());
    let position = quantile * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f64;
    sorted[lower] + (sorted[upper] - sorted[lower]) * fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsr_reference_profile_fails_exactly_ten_percent_exceedance() {
        let target = vec![Some(100.0); 10];
        let mut measured = vec![Some(100.0); 10];
        measured[9] = Some(108.0);

        let result = assess_reference_rsr_tracking(&target, &measured, 10.0)
            .expect("aligned finite power trace should be assessable");

        assert_eq!(result.sample_count, 10);
        assert_eq!(result.threshold_exceedance_count, 1);
        assert_eq!(result.threshold_exceedance_fraction, 0.1);
        assert!(!result.exceedance_rate_requirement_satisfied);
        // FlexDC-Sim checks p90 > 0.3, while the stricter empirical
        // exceedance-rate metric rejects exactly 10%. These are distinct.
        assert!(result.p90_normalized_absolute_error < 0.3);
        assert!(result.p90_threshold_satisfied);
        assert_eq!(result.max_normalized_absolute_error, 0.8);
    }

    #[test]
    fn rsr_reference_profile_accepts_exceedance_below_ten_percent() {
        let target = vec![Some(100.0); 11];
        let mut measured = vec![Some(100.0); 11];
        measured[10] = Some(104.0);

        let result = assess_reference_rsr_tracking(&target, &measured, 10.0)
            .expect("aligned finite power trace should be assessable");

        assert_eq!(result.threshold_exceedance_count, 1);
        assert!(result.threshold_exceedance_fraction < 0.1);
        assert!(result.exceedance_rate_requirement_satisfied);
        assert!(result.p90_threshold_satisfied);
    }

    #[test]
    fn rsr_error_equal_to_threshold_is_not_an_exceedance() {
        let target = vec![Some(100.0); 10];
        let mut measured = vec![Some(100.0); 10];
        measured[9] = Some(103.0);

        let result = assess_reference_rsr_tracking(&target, &measured, 10.0)
            .expect("aligned finite power trace should be assessable");

        assert_eq!(result.normalized_absolute_errors[9], 0.3);
        assert_eq!(result.threshold_exceedance_count, 0);
        assert_eq!(result.threshold_exceedance_fraction, 0.0);
        assert!(result.exceedance_rate_requirement_satisfied);
        assert!(result.p90_threshold_satisfied);
    }

    #[test]
    fn source_normalized_rsr_series_matches_reference_error_quantile_and_strict_rule() {
        let mut errors = vec![Some(0.0); 10];
        errors[9] = Some(-0.8);

        let result = assess_normalized_rsr_tracking_errors(&errors, 0.3, 0.1)
            .expect("finite source-normalized errors should be assessed");

        assert_eq!(result.normalization_basis, "source_normalized_error_series");
        assert_eq!(result.sample_count, 10);
        assert!((result.p90_normalized_absolute_error - 0.08).abs() < 1.0e-12);
        assert_eq!(result.threshold_exceedance_count, 1);
        assert_eq!(result.threshold_exceedance_fraction, 0.1);
        assert!(!result.exceedance_rate_requirement_satisfied);
        assert!(result.p90_threshold_satisfied);
    }

    #[test]
    fn source_normalized_rsr_series_rejects_missing_or_non_finite_values() {
        assert_eq!(
            assess_normalized_rsr_tracking_errors(&[Some(0.0), None], 0.3, 0.1),
            Err(MetricError::MissingObservation {
                series: "signed_normalized_errors",
                index: 1
            })
        );
        assert!(matches!(
            assess_normalized_rsr_tracking_errors(&[Some(f64::NAN)], 0.3, 0.1),
            Err(MetricError::InvalidObservation {
                series: "signed_normalized_errors",
                index: 0,
                ..
            })
        ));
    }

    #[test]
    fn rsr_rejects_missing_samples_instead_of_dropping_them() {
        let target = vec![Some(100.0), None, Some(100.0)];
        let measured = vec![Some(100.0); 3];

        assert_eq!(
            assess_reference_rsr_tracking(&target, &measured, 5.0),
            Err(MetricError::MissingObservation {
                series: "target_power_kw",
                index: 1,
            })
        );
    }

    #[test]
    fn rsr_rejects_zero_reserve_and_misaligned_series() {
        let target = vec![Some(100.0), Some(100.0)];
        let measured = vec![Some(100.0); 2];

        assert!(matches!(
            assess_reference_rsr_tracking(&target, &measured, 0.0),
            Err(MetricError::InvalidScalar {
                field: "reserve_capacity_kw",
                ..
            })
        ));
        assert!(matches!(
            assess_reference_rsr_tracking(&target, &measured[..1], 5.0),
            Err(MetricError::LengthMismatch {
                series: "measured_power_kw",
                ..
            })
        ));
    }

    #[test]
    fn edr_reports_gross_curtailment_rebound_and_net_energy_separately() {
        let baseline = vec![Some(100.0); 3];
        let event = vec![Some(80.0), Some(110.0), Some(70.0)];
        let durations = vec![1.0, 1.0, 1.0];

        let result = assess_edr_curtailment(&baseline, &event, &durations)
            .expect("aligned finite power trace should be assessable");

        assert_eq!(result.baseline_energy_kwh, 300.0);
        assert_eq!(result.event_energy_kwh, 260.0);
        assert_eq!(result.net_energy_reduction_kwh, 40.0);
        assert!((result.net_energy_reduction_fraction.unwrap() - 2.0 / 15.0).abs() < 1.0e-12);
        assert_eq!(result.gross_curtailment_kwh, 50.0);
        assert_eq!(result.rebound_energy_kwh, 10.0);
        assert_eq!(result.rebound_interval_count, 1);
        assert_eq!(result.peak_baseline_power_kw, 100.0);
        assert_eq!(result.peak_event_power_kw, 110.0);
        assert_eq!(result.peak_event_power_excess_over_baseline_kw, 10.0);
    }

    #[test]
    fn edr_zero_baseline_withholds_relative_reduction_fraction() {
        let baseline = vec![Some(0.0); 2];
        let event = vec![Some(0.0); 2];
        let durations = vec![0.5, 1.0];

        let result = assess_edr_curtailment(&baseline, &event, &durations)
            .expect("zero consumption is valid when absolute energies are known");

        assert_eq!(result.baseline_energy_kwh, 0.0);
        assert_eq!(result.net_energy_reduction_kwh, 0.0);
        assert_eq!(result.net_energy_reduction_fraction, None);
    }

    #[test]
    fn edr_rebound_can_exceed_curtailment_and_make_net_reduction_negative() {
        let baseline = vec![Some(100.0), Some(100.0)];
        let event = vec![Some(80.0), Some(150.0)];
        let durations = vec![1.0, 1.0];

        let result = assess_edr_curtailment(&baseline, &event, &durations)
            .expect("aligned finite power trace should be assessable");

        assert_eq!(result.gross_curtailment_kwh, 20.0);
        assert_eq!(result.rebound_energy_kwh, 50.0);
        assert_eq!(result.net_energy_reduction_kwh, -30.0);
        assert_eq!(result.net_energy_reduction_fraction, Some(-0.15));
        assert_eq!(result.peak_event_power_excess_over_baseline_kw, 50.0);
    }

    #[test]
    fn edr_rejects_missing_or_non_physical_power_observations() {
        let baseline = vec![Some(10.0), None];
        let event = vec![Some(9.0), Some(8.0)];
        let durations = vec![1.0, 1.0];
        assert!(matches!(
            assess_edr_curtailment(&baseline, &event, &durations),
            Err(MetricError::MissingObservation {
                series: "baseline_power_kw",
                index: 1
            })
        ));

        let negative = vec![Some(10.0), Some(-1.0)];
        assert!(matches!(
            assess_edr_curtailment(&negative, &event, &durations),
            Err(MetricError::InvalidObservation {
                series: "baseline_power_kw",
                index: 1,
                ..
            })
        ));
    }

    #[test]
    fn rsr_rejects_non_finite_derived_error_from_finite_inputs() {
        let target = vec![Some(1.0e308)];
        let measured = vec![Some(0.0)];

        assert!(matches!(
            assess_reference_rsr_tracking(&target, &measured, 1.0e-308),
            Err(MetricError::InvalidObservation {
                series: "normalized_absolute_error",
                index: 0,
                ..
            })
        ));
    }

    #[test]
    fn edr_rejects_non_finite_derived_energy_from_finite_inputs() {
        let baseline = vec![Some(1.0e308)];
        let event = vec![Some(0.0)];
        let durations = vec![1.0e308];

        assert!(matches!(
            assess_edr_curtailment(&baseline, &event, &durations),
            Err(MetricError::InvalidObservation {
                series: "derived_energy_kwh",
                index: 0,
                ..
            })
        ));
    }

    #[test]
    fn edr_rejects_misaligned_durations() {
        let baseline = vec![Some(10.0); 2];
        let event = vec![Some(9.0); 2];
        assert!(matches!(
            assess_edr_curtailment(&baseline, &event, &[1.0]),
            Err(MetricError::LengthMismatch {
                series: "interval_duration_hours",
                ..
            })
        ));
    }
}
