// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Matched baseline runner for the abstract Earth Observability profile fixture.
//!
//! It compares sparse training-transect borehole interpolation with available
//! blind-transect scalar geophysical proxy observations against independent
//! held-out truth. These are synthetic software benchmarks, not physical data.

use serde::{Deserialize, Serialize};
use sol_atlas_core::earth_observability_synthetic::{
    SyntheticObservationState, SyntheticProfileFixture,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

const DEFAULT_FIXTURE: &str =
    "research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json";
const DEFAULT_REPORT: &str =
    "research/earth-observability/baselines/synthetic-profile-baseline-v0.1.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineMetrics {
    prediction_count: usize,
    target_count: usize,
    coverage_fraction: f64,
    mean_absolute_error_m: f64,
    root_mean_squared_error_m: f64,
    signed_bias_m: f64,
    p95_absolute_error_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineResult {
    method: String,
    evaluation_site: String,
    metrics: BaselineMetrics,
    abstained_positions_m: Vec<f64>,
    interpretation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineReport {
    benchmark_id: String,
    fixture_id: String,
    fixture_version: String,
    seed: u64,
    status: String,
    geography: String,
    claim_boundary: String,
    target: String,
    results: Vec<BaselineResult>,
    comparison_warning: String,
}

fn ordered_samples(samples: &[(f64, f64)]) -> Option<Vec<(f64, f64)>> {
    if samples.is_empty()
        || samples
            .iter()
            .any(|sample| !sample.0.is_finite() || !sample.1.is_finite())
    {
        return None;
    }

    let mut ordered = samples.to_vec();
    ordered.sort_by(|left, right| left.0.total_cmp(&right.0));
    if ordered.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return None;
    }
    Some(ordered)
}

fn interpolate_piecewise_linear(
    samples: &[(f64, f64)],
    target_x_m: f64,
) -> Option<f64> {
    if !target_x_m.is_finite() {
        return None;
    }
    let ordered = ordered_samples(samples)?;

    if target_x_m < ordered[0].0 || target_x_m > ordered[ordered.len() - 1].0 {
        return None;
    }

    if let Some((_, value)) = ordered.iter().find(|(x_m, _)| *x_m == target_x_m) {
        return Some(*value);
    }

    for pair in ordered.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if target_x_m > x0 && target_x_m < x1 {
            let fraction = (target_x_m - x0) / (x1 - x0);
            return Some(y0 + fraction * (y1 - y0));
        }
    }

    None
}

fn nearest_neighbor(samples: &[(f64, f64)], target_x_m: f64) -> Option<f64> {
    if !target_x_m.is_finite() {
        return None;
    }
    let ordered = ordered_samples(samples)?;
    if target_x_m < ordered[0].0 || target_x_m > ordered[ordered.len() - 1].0 {
        return None;
    }

    ordered
        .iter()
        .min_by(|left, right| {
            (left.0 - target_x_m)
                .abs()
                .total_cmp(&(right.0 - target_x_m).abs())
                .then_with(|| left.0.total_cmp(&right.0))
        })
        .map(|sample| sample.1)
}

fn inverse_distance_weighted(
    samples: &[(f64, f64)],
    target_x_m: f64,
    power: f64,
) -> Option<f64> {
    if !target_x_m.is_finite() || !power.is_finite() || power <= 0.0 {
        return None;
    }
    let ordered = ordered_samples(samples)?;
    if target_x_m < ordered[0].0 || target_x_m > ordered[ordered.len() - 1].0 {
        return None;
    }
    if let Some((_, value)) = ordered.iter().find(|(x_m, _)| *x_m == target_x_m) {
        return Some(*value);
    }

    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;
    for (x_m, value) in &ordered {
        let distance = (target_x_m - x_m).abs();
        if distance == 0.0 {
            return Some(*value);
        }
        let weight = 1.0 / distance.powf(power);
        weighted_sum += value * weight;
        weight_sum += weight;
    }

    if !weight_sum.is_finite() || weight_sum <= 0.0 {
        None
    } else {
        Some(weighted_sum / weight_sum)
    }
}

fn metrics(
    predictions: &[(f64, f64)],
    target_count: usize,
) -> Result<BaselineMetrics, String> {
    if predictions.is_empty() || target_count == 0 || predictions.len() > target_count {
        return Err("invalid prediction/target count".to_string());
    }
    if predictions
        .iter()
        .any(|(prediction, actual)| !prediction.is_finite() || !actual.is_finite())
    {
        return Err("non-finite prediction or reference value".to_string());
    }

    let errors: Vec<f64> = predictions
        .iter()
        .map(|(prediction, actual)| prediction - actual)
        .collect();
    let mut absolute_errors: Vec<f64> = errors.iter().map(|error| error.abs()).collect();
    absolute_errors.sort_by(f64::total_cmp);

    let count = errors.len() as f64;
    let mae = absolute_errors.iter().sum::<f64>() / count;
    let mse = errors.iter().map(|error| error * error).sum::<f64>() / count;
    let bias = errors.iter().sum::<f64>() / count;
    let p95_index = ((0.95 * count).ceil() as usize)
        .saturating_sub(1)
        .min(absolute_errors.len() - 1);

    Ok(BaselineMetrics {
        prediction_count: predictions.len(),
        target_count,
        coverage_fraction: predictions.len() as f64 / target_count as f64,
        mean_absolute_error_m: mae,
        root_mean_squared_error_m: mse.sqrt(),
        signed_bias_m: bias,
        p95_absolute_error_m: absolute_errors[p95_index],
    })
}

fn run_baseline(fixture: &SyntheticProfileFixture) -> Result<BaselineReport, String> {
    if fixture.geography != "abstract_non_geographic"
        || fixture.status != "synthetic_generated_not_field_data"
    {
        return Err("fixture claim boundary is not an admitted synthetic-only profile".to_string());
    }

    let blind_site = "synthetic-blind-transect";
    let blind_truth: Vec<_> = fixture
        .independent_validation_truth
        .iter()
        .filter(|truth| truth.site_id == blind_site)
        .collect();
    if blind_truth.is_empty() {
        return Err("independent blind-transect truth is missing".to_string());
    }

    let training_boreholes: Vec<(f64, f64)> = fixture
        .borehole_observations
        .iter()
        .filter(|sample| {
            sample.site_id == "synthetic-training-transect"
                && sample.split_role
                    == sol_atlas_core::earth_observability_synthetic::SyntheticSplitRole::Fit
        })
        .map(|sample| (sample.x_m, sample.interface_depth_m))
        .collect();

    let mut borehole_predictions = Vec::new();
    let mut borehole_abstentions = Vec::new();
    let mut nearest_predictions = Vec::new();
    let mut nearest_abstentions = Vec::new();
    let mut idw_predictions = Vec::new();
    let mut idw_abstentions = Vec::new();
    for truth in &blind_truth {
        match interpolate_piecewise_linear(&training_boreholes, truth.x_m) {
            Some(prediction) => {
                borehole_predictions.push((prediction, truth.true_interface_depth_m));
            }
            None => borehole_abstentions.push(truth.x_m),
        }
        match nearest_neighbor(&training_boreholes, truth.x_m) {
            Some(prediction) => {
                nearest_predictions.push((prediction, truth.true_interface_depth_m));
            }
            None => nearest_abstentions.push(truth.x_m),
        }
        match inverse_distance_weighted(&training_boreholes, truth.x_m, 2.0) {
            Some(prediction) => {
                idw_predictions.push((prediction, truth.true_interface_depth_m));
            }
            None => idw_abstentions.push(truth.x_m),
        }
    }

    let truth_by_x: BTreeMap<u64, f64> = blind_truth
        .iter()
        .map(|truth| (truth.x_m.to_bits(), truth.true_interface_depth_m))
        .collect();

    let mut proxy_predictions = Vec::new();
    let mut proxy_positions = Vec::new();
    for observation in fixture.geophysical_observations.iter().filter(|observation| {
        observation.site_id == blind_site
            && observation.state == SyntheticObservationState::Present
    }) {
        let Some(prediction) = observation.interface_depth_estimate_m else {
            return Err(format!(
                "present proxy {} has no estimate",
                observation.observation_id
            ));
        };
        let Some(actual) = truth_by_x.get(&observation.x_m.to_bits()) else {
            return Err(format!(
                "proxy {} has no independent truth at its support",
                observation.observation_id
            ));
        };
        proxy_predictions.push((prediction, *actual));
        proxy_positions.push(observation.x_m);
    }

    let proxy_abstentions: Vec<f64> = blind_truth
        .iter()
        .filter(|truth| !proxy_positions.contains(&truth.x_m))
        .map(|truth| truth.x_m)
        .collect();

    Ok(BaselineReport {
        benchmark_id: "EARTH-OBSERVABILITY-SYNTHETIC-BASELINES-001".to_string(),
        fixture_id: fixture.fixture_id.clone(),
        fixture_version: fixture.fixture_version.clone(),
        seed: fixture.seed,
        status: "synthetic_baseline_result_not_field_evidence".to_string(),
        geography: "abstract_non_geographic".to_string(),
        claim_boundary: "Synthetic profile benchmark only; scalar interface proxy values are not physical sensor measurements, waveforms, geological truth or engineering properties.".to_string(),
        target: "interface_depth_m on the held-out synthetic transect".to_string(),
        results: vec![
            BaselineResult {
                method: "training_borehole_piecewise_linear_cross_transect".to_string(),
                evaluation_site: blind_site.to_string(),
                metrics: metrics(&borehole_predictions, blind_truth.len())?,
                abstained_positions_m: borehole_abstentions,
                interpretation: "Uses only sparse training-transect borehole contacts; evaluates transfer to the shifted blind transect. No blind truth or blind proxy is used to fit this baseline.".to_string(),
            },
            BaselineResult {
                method: "training_borehole_nearest_neighbor_cross_transect".to_string(),
                evaluation_site: blind_site.to_string(),
                metrics: metrics(&nearest_predictions, blind_truth.len())?,
                abstained_positions_m: nearest_abstentions,
                interpretation: "Uses only sparse training-transect borehole contacts and selects the closest contact, breaking equal-distance ties toward the lower x coordinate. No blind truth or blind proxy is used to fit this baseline.".to_string(),
            },
            BaselineResult {
                method: "training_borehole_idw_power_2_cross_transect".to_string(),
                evaluation_site: blind_site.to_string(),
                metrics: metrics(&idw_predictions, blind_truth.len())?,
                abstained_positions_m: idw_abstentions,
                interpretation: "Uses only sparse training-transect borehole contacts with inverse-distance-squared weights; no variogram is fitted and this is not kriging. No blind truth or blind proxy is used to fit this baseline.".to_string(),
            },
            BaselineResult {
                method: "blind_transect_surface_wave_interface_proxy_only".to_string(),
                evaluation_site: blind_site.to_string(),
                metrics: metrics(&proxy_predictions, blind_truth.len())?,
                abstained_positions_m: proxy_abstentions,
                interpretation: "Scores only observed scalar proxy positions on the blind transect; unobserved positions abstain and are not zero-filled. The proxy is synthetic and does not model sensor physics.".to_string(),
            },
        ],
        comparison_warning: "The methods have different observation coverage and represent a synthetic transfer/observability stress test, not a matched physical field trial. Report accuracy together with coverage and abstentions; do not infer that one physical modality is superior.".to_string(),
    })
}

fn load_fixture(path: &str) -> Result<SyntheticProfileFixture, String> {
    let bytes = fs::read(Path::new(path))
        .map_err(|error| format!("cannot read {path}: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("fixture parse failed: {error}"))
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} [FIXTURE.json]");
    eprintln!("       {program} --check REPORT.json [FIXTURE.json]");
    eprintln!("Default fixture: {DEFAULT_FIXTURE}");
}

fn main() {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "earth_observability_baseline".to_string());

    let first = args.next();
    let (check_path, fixture_path) = match first.as_deref() {
        None => (None, DEFAULT_FIXTURE.to_string()),
        Some("--help" | "-h") => {
            print_usage(&program);
            return;
        }
        Some("--check") => {
            let Some(report_path) = args.next() else {
                print_usage(&program);
                std::process::exit(64);
            };
            let fixture_path = args.next().unwrap_or_else(|| DEFAULT_FIXTURE.to_string());
            (Some(report_path), fixture_path)
        }
        Some(path) => (None, path.to_string()),
    };

    if args.next().is_some() {
        print_usage(&program);
        std::process::exit(64);
    }

    let report = load_fixture(&fixture_path).and_then(|fixture| run_baseline(&fixture));
    match report {
        Ok(report) => {
            if let Some(report_path) = check_path {
                let expected_bytes = match fs::read(Path::new(&report_path)) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        eprintln!("cannot read {report_path}: {error}");
                        std::process::exit(1);
                    }
                };
                let expected: BaselineReport = match serde_json::from_slice(&expected_bytes) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("baseline report parse failed: {error}");
                        std::process::exit(1);
                    }
                };
                if expected != report {
                    eprintln!("FAIL: committed baseline report differs from recomputed metrics");
                    std::process::exit(2);
                }
                println!(
                    "PASS: {report_path} matches fixture {} seed {}",
                    report.fixture_id, report.seed
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .expect("baseline report must serialize")
                );
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json"
    );

    #[test]
    fn piecewise_interpolation_does_not_extrapolate() {
        let samples = vec![(0.0, 1.0), (10.0, 3.0), (20.0, 5.0)];
        assert_eq!(interpolate_piecewise_linear(&samples, 5.0), Some(2.0));
        assert_eq!(interpolate_piecewise_linear(&samples, 25.0), None);
    }

    #[test]
    fn nearest_neighbor_breaks_ties_toward_lower_coordinate() {
        let samples = vec![(20.0, 4.0), (0.0, 2.0)];
        assert_eq!(nearest_neighbor(&samples, 10.0), Some(2.0));
    }

    #[test]
    fn idw_is_exact_at_samples_and_does_not_extrapolate() {
        let samples = vec![(0.0, 1.0), (10.0, 3.0), (20.0, 5.0)];
        assert_eq!(inverse_distance_weighted(&samples, 10.0, 2.0), Some(3.0));
        assert_eq!(inverse_distance_weighted(&samples, 25.0, 2.0), None);
        assert_eq!(inverse_distance_weighted(&samples, 5.0, 0.0), None);
    }

    #[test]
    fn duplicate_or_non_finite_interpolation_support_is_rejected() {
        assert_eq!(
            interpolate_piecewise_linear(&[(0.0, 1.0), (0.0, 2.0)], 0.0),
            None
        );
        assert_eq!(
            interpolate_piecewise_linear(&[(0.0, f64::NAN), (10.0, 2.0)], 5.0),
            None
        );
    }

    #[test]
    fn blind_borehole_transfer_metrics_are_reproducible() {
        let fixture: SyntheticProfileFixture =
            serde_json::from_str(FIXTURE).expect("fixture parses");
        let report = run_baseline(&fixture).expect("baseline runs");
        let borehole = &report.results[0].metrics;
        assert_eq!(borehole.prediction_count, 11);
        assert_eq!(borehole.target_count, 11);
        assert!((borehole.mean_absolute_error_m - 1.2518181818181817).abs() < 1e-12);
        assert!((borehole.signed_bias_m + 1.2518181818181817).abs() < 1e-12);
        let nearest = &report.results[1].metrics;
        assert!((nearest.mean_absolute_error_m - 1.4263636363636363).abs() < 1e-12);
        let idw = &report.results[2].metrics;
        assert!((idw.mean_absolute_error_m - 1.2539003805590936).abs() < 1e-12);
    }

    #[test]
    fn proxy_baseline_reports_coverage_and_preserves_abstentions() {
        let fixture: SyntheticProfileFixture =
            serde_json::from_str(FIXTURE).expect("fixture parses");
        let report = run_baseline(&fixture).expect("baseline runs");
        let proxy = &report.results[1];
        assert_eq!(proxy.metrics.prediction_count, 6);
        assert_eq!(proxy.metrics.target_count, 11);
        assert!((proxy.metrics.coverage_fraction - 6.0 / 11.0).abs() < 1e-12);
        assert_eq!(proxy.abstained_positions_m, vec![60.0, 70.0, 80.0, 90.0, 100.0]);
        assert!((proxy.metrics.mean_absolute_error_m - 0.045).abs() < 1e-12);
    }

    #[test]
    fn report_cannot_be_promoted_to_field_evidence() {
        let mut fixture: SyntheticProfileFixture =
            serde_json::from_str(FIXTURE).expect("fixture parses");
        fixture.status = "field_measurements".to_string();
        assert!(run_baseline(&fixture).is_err());
    }
}
