// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reproducible consistency audit for operator-published CHARGE N3 trip summaries.
//!
//! This is an audit of co-published aggregate fields, not raw telemetry,
//! independent vehicle-energy validation, a calibrated predictor or an
//! infrastructure availability claim.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const DATASET_SCHEMA_VERSION: &str = "n3-published-trip-summaries/v0.1";
pub const REPORT_SCHEMA_VERSION: &str = "n3-published-trip-audit-report/v0.1";
pub const ABSOLUTE_ENERGY_TOLERANCE_KWH: f64 = 0.10;
pub const RELATIVE_ENERGY_TOLERANCE_PERCENT: f64 = 0.50;
pub const RANGE_TOLERANCE_KM: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedTrip {
    pub trip_id: String,
    pub test_date: String,
    pub direction: Direction,
    pub vehicle_make: String,
    pub vehicle_model: String,
    #[serde(default)]
    pub reported_pack_capacity_kwh: Option<f64>,
    pub distance_km: f64,
    pub elevation_change_m: i64,
    pub total_ascent_m: u64,
    pub total_descent_m: u64,
    pub average_speed_kmh: f64,
    pub battery_used_percent: f64,
    #[serde(default)]
    pub energy_efficiency_kwh_per_100km: Option<f64>,
    #[serde(default)]
    pub published_range_km_per_100pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedDataset {
    pub schema_version: String,
    pub dataset_id: String,
    pub publisher: String,
    pub source_url: String,
    pub retrieved_at_utc_date: String,
    pub evidence_class: String,
    pub raw_telemetry_included: bool,
    pub limitations: Vec<String>,
    pub trips: Vec<PublishedTrip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TripAssessment {
    pub trip_id: String,
    pub direction: Direction,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub distance_km: f64,
    pub elevation_change_m: i64,
    pub average_speed_kmh: f64,
    pub published_energy_efficiency_kwh_per_100km: Option<f64>,
    pub energy_from_published_efficiency_kwh: Option<f64>,
    pub nominal_capacity_soc_proxy_kwh: Option<f64>,
    pub absolute_energy_residual_kwh: Option<f64>,
    pub relative_energy_residual_percent: Option<f64>,
    pub energy_consistency: String,
    pub published_range_km_per_100pct: Option<f64>,
    pub range_from_battery_used_km: Option<f64>,
    pub absolute_range_residual_km: Option<f64>,
    pub range_consistency: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupSummary {
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub direction: Direction,
    pub sample_count: usize,
    pub minimum_kwh_per_100km: f64,
    pub median_kwh_per_100km: f64,
    pub maximum_kwh_per_100km: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub schema_version: String,
    pub dataset_id: String,
    pub source_url: String,
    pub retrieved_at_utc_date: String,
    pub evidence_class: String,
    pub raw_telemetry_included: bool,
    pub input_sha256: String,
    pub report_sha256: String,
    pub trip_count: usize,
    pub energy_comparison_count: usize,
    pub energy_within_published_rounding_count: usize,
    pub energy_comparison_exceeds_rounding_count: usize,
    pub energy_comparison_unavailable_count: usize,
    pub range_within_published_rounding_count: usize,
    pub maximum_absolute_energy_residual_kwh: f64,
    pub maximum_relative_energy_residual_percent: f64,
    pub group_summaries: Vec<GroupSummary>,
    pub trips: Vec<TripAssessment>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEnvelope {
    pub report: AuditReport,
}

pub fn audit_json(input_bytes: &[u8]) -> Result<AuditEnvelope, String> {
    let dataset: PublishedDataset = serde_json::from_slice(input_bytes)
        .map_err(|error| format!("published N3 trip dataset JSON is invalid: {error}"))?;
    let report = audit_dataset(&dataset, input_bytes)?;
    Ok(AuditEnvelope { report })
}

pub fn audit_dataset(
    dataset: &PublishedDataset,
    input_bytes: &[u8],
) -> Result<AuditReport, String> {
    if dataset.schema_version != DATASET_SCHEMA_VERSION {
        return Err(format!(
            "unsupported dataset schema version; expected {DATASET_SCHEMA_VERSION}"
        ));
    }
    require_text(&dataset.dataset_id, "dataset_id")?;
    require_text(&dataset.publisher, "publisher")?;
    require_text(&dataset.source_url, "source_url")?;
    require_date(&dataset.retrieved_at_utc_date, "retrieved_at_utc_date")?;
    if dataset.publisher != "CHARGE N3" {
        return Err("publisher must be CHARGE N3 for this source-specific v0.1 corpus".into());
    }
    if !dataset
        .source_url
        .starts_with("https://charge.co.za/n3-real-world-ev-tests/")
    {
        return Err("source_url must identify the official CHARGE N3 test results page".into());
    }
    if dataset.evidence_class != "operator_published_summary" {
        return Err(
            "evidence_class must be operator_published_summary for this v0.1 corpus".into(),
        );
    }
    if dataset.raw_telemetry_included {
        return Err("v0.1 public-summary corpus must not claim to contain raw telemetry".into());
    }
    if dataset.limitations.is_empty() {
        return Err("limitations must state the evidence/claim boundary".into());
    }
    if dataset.trips.is_empty() {
        return Err("dataset must contain at least one published trip summary".into());
    }

    let mut trip_ids = BTreeSet::new();
    let mut assessments = Vec::with_capacity(dataset.trips.len());
    let mut grouped: BTreeMap<(String, String, Direction), Vec<f64>> = BTreeMap::new();
    let mut energy_comparison_count = 0usize;
    let mut energy_pass_count = 0usize;
    let mut energy_comparison_exceeds_rounding_count = 0usize;
    let mut energy_comparison_unavailable_count = 0usize;
    let mut range_pass_count = 0usize;
    let mut max_abs_energy_residual = 0.0_f64;
    let mut max_relative_energy_residual = 0.0_f64;

    for trip in &dataset.trips {
        require_text(&trip.trip_id, "trips[].trip_id")?;
        if !trip_ids.insert(trip.trip_id.clone()) {
            return Err(format!("duplicate trip_id: {}", trip.trip_id));
        }
        require_date(&trip.test_date, &format!("{}.test_date", trip.trip_id))?;
        require_text(
            &trip.vehicle_make,
            &format!("{}.vehicle_make", trip.trip_id),
        )?;
        require_text(
            &trip.vehicle_model,
            &format!("{}.vehicle_model", trip.trip_id),
        )?;
        if let Some(capacity) = trip.reported_pack_capacity_kwh {
            validate_positive(capacity, "reported_pack_capacity_kwh", &trip.trip_id)?;
        }
        validate_positive(trip.distance_km, "distance_km", &trip.trip_id)?;
        validate_positive(trip.average_speed_kmh, "average_speed_kmh", &trip.trip_id)?;
        if let Some(efficiency) = trip.energy_efficiency_kwh_per_100km {
            validate_positive(efficiency, "energy_efficiency_kwh_per_100km", &trip.trip_id)?;
        }
        validate_finite_nonnegative(
            trip.battery_used_percent,
            "battery_used_percent",
            &trip.trip_id,
        )?;
        if trip.battery_used_percent <= 0.0 || trip.battery_used_percent > 100.0 {
            return Err(format!(
                "{}: battery_used_percent must be in (0, 100]",
                trip.trip_id
            ));
        }
        if trip.average_speed_kmh > 200.0 {
            return Err(format!(
                "{}: average_speed_kmh exceeds the v0.1 plausibility bound",
                trip.trip_id
            ));
        }
        if trip.total_ascent_m == 0 || trip.total_descent_m == 0 {
            return Err(format!(
                "{}: total ascent and descent must both be present",
                trip.trip_id
            ));
        }
        let net_elevation = i128::from(trip.total_ascent_m) - i128::from(trip.total_descent_m);
        if (net_elevation - i128::from(trip.elevation_change_m)).abs() > 5 {
            return Err(format!(
                "{}: ascent minus descent does not reconcile to published elevation_change_m within 5 m",
                trip.trip_id
            ));
        }
        match trip.direction {
            Direction::Up if trip.elevation_change_m <= 0 => {
                return Err(format!(
                    "{}: an up trip must have positive net elevation change",
                    trip.trip_id
                ));
            }
            Direction::Down if trip.elevation_change_m >= 0 => {
                return Err(format!(
                    "{}: a down trip must have negative net elevation change",
                    trip.trip_id
                ));
            }
            _ => {}
        }
        if let Some(range) = trip.published_range_km_per_100pct {
            validate_positive(range, "published_range_km_per_100pct", &trip.trip_id)?;
        }

        // These co-published fields may share an upstream calculation. We
        // compare them only when both are present; absence is tracked as
        // coverage, never imputed or silently treated as a passing result.
        let energy_from_efficiency = trip
            .energy_efficiency_kwh_per_100km
            .map(|efficiency| efficiency * trip.distance_km / 100.0);
        let capacity_soc_proxy = trip
            .reported_pack_capacity_kwh
            .map(|capacity| capacity * trip.battery_used_percent / 100.0);
        if energy_from_efficiency.is_some_and(|value| !value.is_finite())
            || capacity_soc_proxy.is_some_and(|value| !value.is_finite())
        {
            return Err(format!("{}: derived energy proxy overflowed", trip.trip_id));
        }
        let (abs_energy_residual, relative_energy_residual, energy_consistency) =
            match (energy_from_efficiency, capacity_soc_proxy) {
                (Some(energy), Some(capacity_proxy)) => {
                    energy_comparison_count += 1;
                    let absolute = (energy - capacity_proxy).abs();
                    let relative = absolute / energy.max(f64::MIN_POSITIVE) * 100.0;
                    if !absolute.is_finite() || !relative.is_finite() {
                        return Err(format!(
                            "{}: derived energy residual is non-finite",
                            trip.trip_id
                        ));
                    }
                    max_abs_energy_residual = max_abs_energy_residual.max(absolute);
                    max_relative_energy_residual = max_relative_energy_residual.max(relative);
                    let consistency = if absolute <= ABSOLUTE_ENERGY_TOLERANCE_KWH
                        && relative <= RELATIVE_ENERGY_TOLERANCE_PERCENT
                    {
                        energy_pass_count += 1;
                        "WITHIN_PUBLISHED_ROUNDING"
                    } else {
                        energy_comparison_exceeds_rounding_count += 1;
                        "EXCEEDS_PUBLISHED_ROUNDING"
                    };
                    (Some(absolute), Some(relative), consistency.to_owned())
                }
                _ => {
                    energy_comparison_unavailable_count += 1;
                    (None, None, "UNAVAILABLE_MISSING_SOURCE_DATA".to_owned())
                }
            };

        let range_from_used = trip.distance_km / (trip.battery_used_percent / 100.0);
        if !range_from_used.is_finite() {
            return Err(format!(
                "{}: derived full-charge range proxy is non-finite",
                trip.trip_id
            ));
        }
        let (range_residual, range_consistency) = match trip.published_range_km_per_100pct {
            Some(published) => {
                let residual = (range_from_used - published).abs();
                let consistency = if residual <= RANGE_TOLERANCE_KM {
                    range_pass_count += 1;
                    "WITHIN_PUBLISHED_ROUNDING".to_owned()
                } else {
                    "EXCEEDS_PUBLISHED_ROUNDING".to_owned()
                };
                (Some(residual), Some(consistency))
            }
            None => (None, None),
        };

        if let Some(efficiency) = trip.energy_efficiency_kwh_per_100km {
            grouped
                .entry((
                    trip.vehicle_make.clone(),
                    trip.vehicle_model.clone(),
                    trip.direction,
                ))
                .or_default()
                .push(efficiency);
        }

        assessments.push(TripAssessment {
            trip_id: trip.trip_id.clone(),
            direction: trip.direction,
            vehicle_make: trip.vehicle_make.clone(),
            vehicle_model: trip.vehicle_model.clone(),
            distance_km: trip.distance_km,
            elevation_change_m: trip.elevation_change_m,
            average_speed_kmh: trip.average_speed_kmh,
            published_energy_efficiency_kwh_per_100km: trip.energy_efficiency_kwh_per_100km,
            energy_from_published_efficiency_kwh: energy_from_efficiency,
            nominal_capacity_soc_proxy_kwh: capacity_soc_proxy,
            absolute_energy_residual_kwh: abs_energy_residual,
            relative_energy_residual_percent: relative_energy_residual,
            energy_consistency,
            published_range_km_per_100pct: trip.published_range_km_per_100pct,
            range_from_battery_used_km: trip.published_range_km_per_100pct.map(|_| range_from_used),
            absolute_range_residual_km: range_residual,
            range_consistency,
        });
    }

    let group_summaries = grouped
        .into_iter()
        .map(|((vehicle_make, vehicle_model, direction), mut values)| {
            values.sort_by(f64::total_cmp);
            let median = if values.len() % 2 == 0 {
                let high = values.len() / 2;
                (values[high - 1] + values[high]) / 2.0
            } else {
                values[values.len() / 2]
            };
            GroupSummary {
                vehicle_make,
                vehicle_model,
                direction,
                sample_count: values.len(),
                minimum_kwh_per_100km: values[0],
                median_kwh_per_100km: median,
                maximum_kwh_per_100km: values[values.len() - 1],
            }
        })
        .collect::<Vec<_>>();

    let input_sha256 = sha256_hex(input_bytes);
    let mut report = AuditReport {
        schema_version: REPORT_SCHEMA_VERSION.into(),
        dataset_id: dataset.dataset_id.clone(),
        source_url: dataset.source_url.clone(),
        retrieved_at_utc_date: dataset.retrieved_at_utc_date.clone(),
        evidence_class: dataset.evidence_class.clone(),
        raw_telemetry_included: dataset.raw_telemetry_included,
        input_sha256,
        report_sha256: String::new(),
        trip_count: assessments.len(),
        energy_comparison_count,
        energy_within_published_rounding_count: energy_pass_count,
        energy_comparison_exceeds_rounding_count,
        energy_comparison_unavailable_count,
        range_within_published_rounding_count: range_pass_count,
        maximum_absolute_energy_residual_kwh: max_abs_energy_residual,
        maximum_relative_energy_residual_percent: max_relative_energy_residual,
        group_summaries,
        trips: assessments,
        claim_ceiling: "Public-summary consistency audit only. The checked fields are co-published aggregate values, not independent instruments. This report does not validate raw telemetry, calibrate an EV energy model, establish battery usable capacity, reproduce a charger power curve, prove route safety, or establish site/charger availability.".into(),
    };
    let canonical_without_hash = serde_json::to_vec(&report)
        .map_err(|error| format!("could not serialize N3 audit report: {error}"))?;
    report.report_sha256 = sha256_hex(&canonical_without_hash);
    Ok(report)
}

fn require_text(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    Ok(())
}

fn require_date(value: &str, label: &str) -> Result<(), String> {
    let valid = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !valid {
        return Err(format!("{label} must be YYYY-MM-DD"));
    }
    Ok(())
}

fn validate_positive(value: f64, label: &str, trip_id: &str) -> Result<(), String> {
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("{trip_id}: {label} must be finite and positive"));
    }
    Ok(())
}

fn validate_finite_nonnegative(value: f64, label: &str, trip_id: &str) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err(format!(
            "{trip_id}: {label} must be finite and non-negative"
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] =
        include_bytes!("../../data/benchmarks/n3_published_trip_summaries_v0_1.json");

    #[test]
    fn published_n3_trip_summaries_preserve_matched_missing_and_mismatched_rows() {
        let envelope = audit_json(FIXTURE).expect("published N3 summary fixture");
        let report = envelope.report;
        assert_eq!(report.trip_count, 26);
        assert_eq!(report.energy_comparison_count, 24);
        assert_eq!(report.energy_within_published_rounding_count, 23);
        assert_eq!(report.energy_comparison_exceeds_rounding_count, 1);
        assert_eq!(report.energy_comparison_unavailable_count, 2);
        assert_eq!(report.range_within_published_rounding_count, 26);
        assert!((report.maximum_absolute_energy_residual_kwh - 2.395).abs() < 1e-9);
        assert!((report.maximum_relative_energy_residual_percent - 7.544257544257543).abs() < 1e-9);
        assert_eq!(report.input_sha256.len(), 64);
        assert_eq!(report.report_sha256.len(), 64);
        assert!(!report.raw_telemetry_included);
    }

    #[test]
    fn i_caur_uphill_energy_fields_remain_a_visible_source_discrepancy() {
        let report = audit_json(FIXTURE).expect("published source data audit");
        let trip = report
            .report
            .trips
            .iter()
            .find(|trip| trip.trip_id == "icaur-v23-up-2026-09-24")
            .expect("iCAUR V23 uphill run");
        assert_eq!(trip.energy_consistency, "EXCEEDS_PUBLISHED_ROUNDING");
        assert!((trip.absolute_energy_residual_kwh.unwrap() - 2.395).abs() < 1e-9);
        assert!(trip.relative_energy_residual_percent.unwrap() > 7.5);
    }

    #[test]
    fn direction_and_elevation_sign_must_agree() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[0].direction = Direction::Up;
        assert!(audit_dataset(&dataset, FIXTURE).is_err());
    }

    #[test]
    fn duplicate_public_trip_ids_are_rejected() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[1].trip_id = dataset.trips[0].trip_id.clone();
        assert!(audit_dataset(&dataset, FIXTURE).is_err());
    }

    #[test]
    fn inconsistent_published_energy_is_reported_not_silently_corrected() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[0].energy_efficiency_kwh_per_100km =
            Some(dataset.trips[0].energy_efficiency_kwh_per_100km.unwrap() * 1.2);
        let report = audit_dataset(&dataset, FIXTURE).expect("structurally valid summary");
        assert!(
            report
                .trips
                .iter()
                .any(|trip| trip.energy_consistency == "EXCEEDS_PUBLISHED_ROUNDING")
        );
    }

    #[test]
    fn range_inconsistency_is_reported_not_rewritten() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[0].published_range_km_per_100pct = Some(700.0);
        let report = audit_dataset(&dataset, FIXTURE).expect("structurally valid summary");
        assert!(report.trips.iter().any(|trip| {
            trip.range_consistency.as_deref() == Some("EXCEEDS_PUBLISHED_ROUNDING")
        }));
    }

    #[test]
    fn derived_energy_overflow_is_rejected() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[0].distance_km = f64::MAX;
        dataset.trips[0].energy_efficiency_kwh_per_100km = Some(f64::MAX);
        assert!(audit_dataset(&dataset, FIXTURE).is_err());
    }

    #[test]
    fn missing_published_metrics_are_reported_as_unavailable_not_imputed() {
        let report = audit_json(FIXTURE).expect("partial source fields remain admissible");
        let geely = report
            .report
            .trips
            .iter()
            .find(|trip| trip.trip_id == "geely-e5-down-2026-09-18")
            .expect("Geely E5 summary");
        assert_eq!(geely.energy_consistency, "UNAVAILABLE_MISSING_SOURCE_DATA");
        assert!(geely.nominal_capacity_soc_proxy_kwh.is_none());

        let dolphin = report
            .report
            .trips
            .iter()
            .find(|trip| trip.trip_id == "byd-dolphin-up-2026-09-06")
            .expect("BYD Dolphin summary");
        assert_eq!(
            dolphin.energy_consistency,
            "UNAVAILABLE_MISSING_SOURCE_DATA"
        );
        assert!(dolphin.published_energy_efficiency_kwh_per_100km.is_none());
    }

    #[test]
    fn invalid_battery_use_is_rejected() {
        let mut dataset: PublishedDataset =
            serde_json::from_slice(FIXTURE).expect("fixture parses");
        dataset.trips[0].battery_used_percent = 101.0;
        assert!(audit_dataset(&dataset, FIXTURE).is_err());
    }
}
