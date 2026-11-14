// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Target-conditional sensor observability contracts and a conservative
//! blind-test scoring kernel.
//!
//! This module evaluates synthetic or admitted detection trials. It does not
//! implement sensor physics, authenticate instrument calibration, establish
//! ground truth, or qualify a real survey. Cluster-aware confidence intervals
//! are deliberately NOT_RUN until an explicit resampling protocol is added.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericRange {
    pub minimum: f64,
    pub maximum: f64,
}

impl NumericRange {
    pub fn validate(&self) -> bool {
        self.minimum.is_finite() && self.maximum.is_finite() && self.minimum <= self.maximum
    }

    pub fn contains(&self, value: f64) -> bool {
        self.validate() && value.is_finite() && value >= self.minimum && value <= self.maximum
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorObservabilityEnvelope {
    pub envelope_id: String,
    pub source_evidence_id: String,
    pub sensor_id: String,
    pub configuration_id: String,
    pub calibration_id: String,
    pub target_class: String,
    pub spatial_support_id: String,
    pub valid_from_unix_s: u64,
    pub valid_until_unix_s: u64,
    pub target_size_m: NumericRange,
    pub target_depth_m: NumericRange,
    /// A normalized, modality-specific contrast index. It is not portable
    /// between sensor modalities without an explicit mapping.
    pub target_contrast: NumericRange,
    /// Every key is a declared environmental covariate and its validated
    /// numeric domain, e.g. moisture_fraction or conductivity_s_m.
    pub environmental_domain: BTreeMap<String, NumericRange>,
    /// Target-specific preregistered POD requirement, not achieved performance.
    pub minimum_pod_requirement: f64,
    /// Maximum false detections per surveyed route kilometre.
    pub maximum_false_alarms_per_km: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionPartition {
    Calibration,
    BlindTest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionTrial {
    pub trial_id: String,
    pub partition: DetectionPartition,
    /// Present only when a reference target exists. A negative control has no
    /// target identity but still declares the target class being searched for.
    pub target_id: Option<String>,
    pub site_id: String,
    pub acquisition_id: String,
    pub observed_at_unix_s: u64,
    pub sensor_id: String,
    pub configuration_id: String,
    pub calibration_id: String,
    pub target_class: String,
    pub target_present: bool,
    pub detector_detected: bool,
    pub target_size_m: Option<f64>,
    pub target_depth_m: Option<f64>,
    pub target_contrast: Option<f64>,
    pub environmental_values: BTreeMap<String, f64>,
    pub surveyed_route_km: f64,
    /// Only valid for a detected, present target; this is detection
    /// localization error, not geological interpretation error.
    pub localization_error_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SensorObservabilityFixture {
    pub benchmark_id: String,
    pub fixture_id: String,
    pub fixture_version: String,
    pub seed: u64,
    pub status: String,
    pub geography: String,
    pub claim_boundary: String,
    pub envelope: SensorObservabilityEnvelope,
    pub trials: Vec<DetectionTrial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainDisposition {
    WithinValidatedEnvelope,
    OutOfDomain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeValidationError {
    EmptyIdentifier(&'static str),
    InvalidTimeWindow,
    InvalidRange(&'static str),
    InvalidEnvironmentalVariable,
    MissingEnvironmentalDomain,
    InvalidPodRequirement,
    InvalidFalseAlarmRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialValidationError {
    EmptyIdentifier(&'static str),
    DuplicateTrialId,
    DuplicateTargetId,
    CrossPartitionSiteLeakage,
    CrossPartitionAcquisitionLeakage,
    CrossPartitionTargetLeakage,
    WrongSensorOrConfiguration,
    WrongCalibration,
    WrongTargetClass,
    TimestampOutsideEnvelope,
    InvalidRouteLength,
    MissingTargetIdentity,
    UnexpectedTargetIdentity,
    MissingTargetCharacteristic(&'static str),
    UnexpectedTargetCharacteristic,
    InvalidTargetCharacteristic,
    MissingEnvironmentalVariable(String),
    InvalidEnvironmentalValue(String),
    MissingLocalizationError,
    UnexpectedLocalizationError,
    InvalidLocalizationError,
    NonBlindEvaluationTrial,
    EmptyFixture,
    InvalidEnvelope,
    MissingCalibrationPartition,
    MissingBlindTestPartition,
    InvalidFixtureClaimBoundary,
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn validate_envelope(envelope: &SensorObservabilityEnvelope) -> Vec<EnvelopeValidationError> {
    let mut errors = Vec::new();

    for (field, value) in [
        ("envelope_id", envelope.envelope_id.as_str()),
        ("source_evidence_id", envelope.source_evidence_id.as_str()),
        ("sensor_id", envelope.sensor_id.as_str()),
        ("configuration_id", envelope.configuration_id.as_str()),
        ("calibration_id", envelope.calibration_id.as_str()),
        ("target_class", envelope.target_class.as_str()),
        ("spatial_support_id", envelope.spatial_support_id.as_str()),
    ] {
        if !non_empty(value) {
            errors.push(EnvelopeValidationError::EmptyIdentifier(field));
        }
    }

    if envelope.valid_from_unix_s >= envelope.valid_until_unix_s {
        errors.push(EnvelopeValidationError::InvalidTimeWindow);
    }
    for (name, range) in [
        ("target_size_m", envelope.target_size_m),
        ("target_depth_m", envelope.target_depth_m),
        ("target_contrast", envelope.target_contrast),
    ] {
        if !range.validate() {
            errors.push(EnvelopeValidationError::InvalidRange(name));
        }
    }
    if envelope.environmental_domain.is_empty() {
        errors.push(EnvelopeValidationError::MissingEnvironmentalDomain);
    }
    for (name, range) in &envelope.environmental_domain {
        if !non_empty(name) || !range.validate() {
            errors.push(EnvelopeValidationError::InvalidEnvironmentalVariable);
        }
    }
    if !envelope.minimum_pod_requirement.is_finite()
        || !(0.0..=1.0).contains(&envelope.minimum_pod_requirement)
    {
        errors.push(EnvelopeValidationError::InvalidPodRequirement);
    }
    if !envelope.maximum_false_alarms_per_km.is_finite()
        || envelope.maximum_false_alarms_per_km < 0.0
    {
        errors.push(EnvelopeValidationError::InvalidFalseAlarmRequirement);
    }

    errors
}

fn classify_domain(
    envelope: &SensorObservabilityEnvelope,
    trial: &DetectionTrial,
) -> Result<DomainDisposition, Vec<TrialValidationError>> {
    let mut errors = Vec::new();

    if trial.sensor_id != envelope.sensor_id || trial.configuration_id != envelope.configuration_id
    {
        errors.push(TrialValidationError::WrongSensorOrConfiguration);
    }
    if trial.calibration_id != envelope.calibration_id {
        errors.push(TrialValidationError::WrongCalibration);
    }
    if trial.target_class != envelope.target_class {
        errors.push(TrialValidationError::WrongTargetClass);
    }
    if trial.observed_at_unix_s < envelope.valid_from_unix_s
        || trial.observed_at_unix_s > envelope.valid_until_unix_s
    {
        errors.push(TrialValidationError::TimestampOutsideEnvelope);
    }
    if !trial.surveyed_route_km.is_finite() || trial.surveyed_route_km <= 0.0 {
        errors.push(TrialValidationError::InvalidRouteLength);
    }

    if trial.target_present {
        if trial.target_id.as_deref().is_none_or(|id| !non_empty(id)) {
            errors.push(TrialValidationError::MissingTargetIdentity);
        }
        for (name, value) in [
            ("target_size_m", trial.target_size_m),
            ("target_depth_m", trial.target_depth_m),
            ("target_contrast", trial.target_contrast),
        ] {
            match value {
                None => errors.push(TrialValidationError::MissingTargetCharacteristic(name)),
                Some(number) if !number.is_finite() || number < 0.0 => {
                    errors.push(TrialValidationError::InvalidTargetCharacteristic)
                }
                Some(_) => {}
            }
        }
    } else {
        if trial.target_id.is_some() {
            errors.push(TrialValidationError::UnexpectedTargetIdentity);
        }
        if trial.target_size_m.is_some()
            || trial.target_depth_m.is_some()
            || trial.target_contrast.is_some()
        {
            errors.push(TrialValidationError::UnexpectedTargetCharacteristic);
        }
    }

    match (
        trial.target_present,
        trial.detector_detected,
        trial.localization_error_m,
    ) {
        (true, true, None) => errors.push(TrialValidationError::MissingLocalizationError),
        (true, true, Some(value)) if !value.is_finite() || value < 0.0 => {
            errors.push(TrialValidationError::InvalidLocalizationError)
        }
        (_, _, Some(_)) if !(trial.target_present && trial.detector_detected) => {
            errors.push(TrialValidationError::UnexpectedLocalizationError)
        }
        (_, _, Some(value)) if !value.is_finite() || value < 0.0 => {
            errors.push(TrialValidationError::InvalidLocalizationError)
        }
        _ => {}
    }

    let mut outside = false;
    for (name, range) in &envelope.environmental_domain {
        match trial.environmental_values.get(name) {
            None => errors.push(TrialValidationError::MissingEnvironmentalVariable(
                name.clone(),
            )),
            Some(value) if !value.is_finite() => errors.push(
                TrialValidationError::InvalidEnvironmentalValue(name.clone()),
            ),
            Some(value) if !range.contains(*value) => outside = true,
            Some(_) => {}
        }
    }

    if trial.target_present {
        if let Some(value) = trial.target_size_m {
            outside |= !envelope.target_size_m.contains(value);
        }
        if let Some(value) = trial.target_depth_m {
            outside |= !envelope.target_depth_m.contains(value);
        }
        if let Some(value) = trial.target_contrast {
            outside |= !envelope.target_contrast.contains(value);
        }
    }

    if errors.is_empty() {
        Ok(if outside {
            DomainDisposition::OutOfDomain
        } else {
            DomainDisposition::WithinValidatedEnvelope
        })
    } else {
        Err(errors)
    }
}

pub fn validate_fixture(
    fixture: &SensorObservabilityFixture,
) -> Result<(), Vec<TrialValidationError>> {
    let mut errors = Vec::new();
    if fixture.trials.is_empty() {
        errors.push(TrialValidationError::EmptyFixture);
    }
    if fixture.status != "synthetic_analytic_detection_trials_not_physical_data"
        || fixture.geography != "abstract_non_geographic"
        || fixture.claim_boundary.trim().len() < 20
    {
        errors.push(TrialValidationError::InvalidFixtureClaimBoundary);
    }

    if !validate_envelope(&fixture.envelope).is_empty() {
        errors.push(TrialValidationError::InvalidEnvelope);
    }

    let mut trial_ids = BTreeSet::new();
    let mut calibration_targets = BTreeSet::new();
    let mut blind_targets = BTreeSet::new();
    let mut has_calibration = false;
    let mut has_blind_test = false;
    let mut calibration_sites = BTreeSet::new();
    let mut blind_sites = BTreeSet::new();
    let mut calibration_acquisitions = BTreeSet::new();
    let mut blind_acquisitions = BTreeSet::new();
    let mut partition_errors = Vec::new();

    for trial in &fixture.trials {
        if !non_empty(&trial.trial_id)
            || !non_empty(&trial.site_id)
            || !non_empty(&trial.acquisition_id)
        {
            errors.push(TrialValidationError::EmptyIdentifier(
                "trial/site/acquisition",
            ));
        }
        if !trial_ids.insert(trial.trial_id.as_str()) {
            errors.push(TrialValidationError::DuplicateTrialId);
        }
        if let Some(target_id) = trial.target_id.as_deref() {
            let inserted = match trial.partition {
                DetectionPartition::Calibration => calibration_targets.insert(target_id),
                DetectionPartition::BlindTest => blind_targets.insert(target_id),
            };
            if !inserted {
                errors.push(TrialValidationError::DuplicateTargetId);
            }
        }
        match trial.partition {
            DetectionPartition::Calibration => {
                has_calibration = true;
                calibration_sites.insert(trial.site_id.as_str());
                calibration_acquisitions.insert(trial.acquisition_id.as_str());
            }
            DetectionPartition::BlindTest => {
                has_blind_test = true;
                blind_sites.insert(trial.site_id.as_str());
                blind_acquisitions.insert(trial.acquisition_id.as_str());
            }
        }
        if let Err(trial_errors) = classify_domain(&fixture.envelope, trial) {
            errors.extend(trial_errors);
        }
    }

    if !has_calibration {
        partition_errors.push(TrialValidationError::MissingCalibrationPartition);
    }
    if !has_blind_test {
        partition_errors.push(TrialValidationError::MissingBlindTestPartition);
    }
    if calibration_targets
        .iter()
        .any(|id| blind_targets.contains(id))
    {
        partition_errors.push(TrialValidationError::CrossPartitionTargetLeakage);
    }
    if calibration_sites
        .iter()
        .any(|site| blind_sites.contains(site))
    {
        partition_errors.push(TrialValidationError::CrossPartitionSiteLeakage);
    }
    if calibration_acquisitions
        .iter()
        .any(|id| blind_acquisitions.contains(id))
    {
        partition_errors.push(TrialValidationError::CrossPartitionAcquisitionLeakage);
    }
    if !partition_errors.is_empty() {
        errors.extend(partition_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteDetectionMetrics {
    pub site_id: String,
    pub positive_targets: usize,
    pub true_positives: usize,
    pub false_negatives: usize,
    pub negative_controls: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub surveyed_negative_route_km: f64,
    pub probability_of_detection: Option<f64>,
    pub false_alarms_per_km: Option<f64>,
}

#[derive(Debug, Default)]
struct SiteAccumulator {
    positive_targets: usize,
    true_positives: usize,
    false_negatives: usize,
    negative_controls: usize,
    false_positives: usize,
    true_negatives: usize,
    surveyed_negative_route_km: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence_level: f64,
    pub independent_site_clusters: usize,
    pub bootstrap_replicates: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterBootstrapConfig {
    pub confidence_level: f64,
    pub bootstrap_replicates: usize,
    /// Set from a preregistered design / power analysis. This is not a
    /// universal statistical standard and must not be silently defaulted.
    pub minimum_independent_site_clusters: usize,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionEvaluationMetrics {
    pub in_domain_blind_trials: usize,
    pub out_of_domain_blind_trials: usize,
    pub positive_targets: usize,
    pub true_positives: usize,
    pub false_negatives: usize,
    pub negative_controls: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub probability_of_detection: Option<f64>,
    pub false_alarms_per_km: Option<f64>,
    pub localization_sample_count: usize,
    pub localization_median_error_m: Option<f64>,
    pub localization_p95_error_m: Option<f64>,
    pub unique_site_clusters: usize,
    pub unique_acquisition_clusters: usize,
    pub per_site: Vec<SiteDetectionMetrics>,
    pub pod_cluster_bootstrap_interval: Option<ClusterConfidenceInterval>,
    pub false_alarm_rate_cluster_bootstrap_interval: Option<ClusterConfidenceInterval>,
    pub clustered_confidence_interval_status: String,
    pub point_estimates_meet_declared_requirements: Option<bool>,
    pub qualification_status: String,
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, length: usize) -> usize {
        (self.next_u64() % length as u64) as usize
    }
}

fn percentile(sorted: &[f64], quantile: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let index = ((quantile * sorted.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    Some(sorted[index])
}

/// Score only blind-test trials. Out-of-domain records are counted but never
/// enter POD, false-alarm or localization denominators. Point estimates are
/// descriptive; no clustered confidence interval or physical qualification is
/// claimed by this function.
pub fn evaluate_blind_trials(
    fixture: &SensorObservabilityFixture,
) -> Result<DetectionEvaluationMetrics, Vec<TrialValidationError>> {
    validate_fixture(fixture)?;

    let mut in_domain = 0usize;
    let mut out_of_domain = 0usize;
    let mut positive_targets = 0usize;
    let mut true_positives = 0usize;
    let mut false_negatives = 0usize;
    let mut negative_controls = 0usize;
    let mut false_positives = 0usize;
    let mut true_negatives = 0usize;
    let mut surveyed_negative_route_km = 0.0;
    let mut localization_errors = Vec::new();
    let mut sites = BTreeSet::new();
    let mut acquisitions = BTreeSet::new();
    let mut per_site: BTreeMap<String, SiteAccumulator> = BTreeMap::new();

    for trial in fixture
        .trials
        .iter()
        .filter(|trial| trial.partition == DetectionPartition::BlindTest)
    {
        if classify_domain(&fixture.envelope, trial)? == DomainDisposition::OutOfDomain {
            out_of_domain += 1;
            continue;
        }

        in_domain += 1;
        sites.insert(trial.site_id.as_str());
        acquisitions.insert(trial.acquisition_id.as_str());
        let site = per_site.entry(trial.site_id.clone()).or_default();

        if trial.target_present {
            site.positive_targets += 1;
            positive_targets += 1;
            if trial.detector_detected {
                true_positives += 1;
                site.true_positives += 1;
                localization_errors.push(
                    trial
                        .localization_error_m
                        .expect("validated detected target has localization error"),
                );
            } else {
                false_negatives += 1;
                site.false_negatives += 1;
            }
        } else {
            negative_controls += 1;
            site.negative_controls += 1;
            surveyed_negative_route_km += trial.surveyed_route_km;
            site.surveyed_negative_route_km += trial.surveyed_route_km;
            if trial.detector_detected {
                false_positives += 1;
                site.false_positives += 1;
            } else {
                true_negatives += 1;
                site.true_negatives += 1;
            }
        }
    }

    localization_errors.sort_by(f64::total_cmp);
    let probability_of_detection =
        (positive_targets > 0).then_some(true_positives as f64 / positive_targets as f64);
    let false_alarms_per_km = (surveyed_negative_route_km > 0.0)
        .then_some(false_positives as f64 / surveyed_negative_route_km);
    let point_estimates_meet_declared_requirements = probability_of_detection
        .zip(false_alarms_per_km)
        .map(|(pod, false_alarms)| {
            pod >= fixture.envelope.minimum_pod_requirement
                && false_alarms <= fixture.envelope.maximum_false_alarms_per_km
        });
    let per_site_metrics = per_site
        .into_iter()
        .map(|(site_id, site)| SiteDetectionMetrics {
            site_id,
            positive_targets: site.positive_targets,
            true_positives: site.true_positives,
            false_negatives: site.false_negatives,
            negative_controls: site.negative_controls,
            false_positives: site.false_positives,
            true_negatives: site.true_negatives,
            surveyed_negative_route_km: site.surveyed_negative_route_km,
            probability_of_detection: (site.positive_targets > 0)
                .then_some(site.true_positives as f64 / site.positive_targets as f64),
            false_alarms_per_km: (site.surveyed_negative_route_km > 0.0)
                .then_some(site.false_positives as f64 / site.surveyed_negative_route_km),
        })
        .collect();

    Ok(DetectionEvaluationMetrics {
        in_domain_blind_trials: in_domain,
        out_of_domain_blind_trials: out_of_domain,
        positive_targets,
        true_positives,
        false_negatives,
        negative_controls,
        false_positives,
        true_negatives,
        probability_of_detection,
        false_alarms_per_km,
        localization_sample_count: localization_errors.len(),
        localization_median_error_m: percentile(&localization_errors, 0.50),
        localization_p95_error_m: percentile(&localization_errors, 0.95),
        unique_site_clusters: sites.len(),
        unique_acquisition_clusters: acquisitions.len(),
        per_site: per_site_metrics,
        pod_cluster_bootstrap_interval: None,
        false_alarm_rate_cluster_bootstrap_interval: None,
        clustered_confidence_interval_status: "NOT_RUN_CLUSTER_AWARE_BOOTSTRAP_REQUIRED"
            .to_string(),
        point_estimates_meet_declared_requirements,
        qualification_status: "NOT_QUALIFIED_CLUSTERED_INTERVALS_NOT_RUN".to_string(),
    })
}

/// Recompute descriptive metrics and, when the preregistered independent-site
/// cluster minimum is met, add percentile cluster-bootstrap intervals for POD
/// and false alarms/km. The two rates are resampled from their own eligible
/// site clusters; this does not create a joint distribution or qualify a
/// physical sensor. The minimum cluster count must come from a design/power
/// analysis for the intended deployment population.
pub fn evaluate_blind_trials_with_cluster_bootstrap(
    fixture: &SensorObservabilityFixture,
    config: ClusterBootstrapConfig,
) -> Result<DetectionEvaluationMetrics, Vec<TrialValidationError>> {
    let mut metrics = evaluate_blind_trials(fixture)?;
    if !config.confidence_level.is_finite()
        || !(0.0..1.0).contains(&config.confidence_level)
        || config.bootstrap_replicates < 100
        || config.minimum_independent_site_clusters == 0
    {
        metrics.clustered_confidence_interval_status =
            "INVALID_BOOTSTRAP_CONFIGURATION_NOT_RUN".to_string();
        return Ok(metrics);
    }

    let pod_clusters: Vec<&SiteDetectionMetrics> = metrics
        .per_site
        .iter()
        .filter(|site| site.positive_targets > 0)
        .collect();
    let false_alarm_clusters: Vec<&SiteDetectionMetrics> = metrics
        .per_site
        .iter()
        .filter(|site| site.surveyed_negative_route_km > 0.0)
        .collect();

    if pod_clusters.len() < config.minimum_independent_site_clusters
        || false_alarm_clusters.len() < config.minimum_independent_site_clusters
    {
        metrics.clustered_confidence_interval_status = format!(
            "INSUFFICIENT_SITE_CLUSTERS_POD_{}_FAR_{}_REQUIRED_{}_NOT_RUN",
            pod_clusters.len(),
            false_alarm_clusters.len(),
            config.minimum_independent_site_clusters
        );
        return Ok(metrics);
    }

    let alpha = 1.0 - config.confidence_level;
    let mut rng = SplitMix64::new(config.seed);
    let mut pod_samples = Vec::with_capacity(config.bootstrap_replicates);
    let mut false_alarm_samples = Vec::with_capacity(config.bootstrap_replicates);

    for _ in 0..config.bootstrap_replicates {
        let mut pod_positive = 0usize;
        let mut pod_hits = 0usize;
        for _ in 0..pod_clusters.len() {
            let cluster = pod_clusters[rng.index(pod_clusters.len())];
            pod_positive += cluster.positive_targets;
            pod_hits += cluster.true_positives;
        }
        if pod_positive > 0 {
            pod_samples.push(pod_hits as f64 / pod_positive as f64);
        }

        let mut negative_route_km = 0.0;
        let mut false_alarms = 0usize;
        for _ in 0..false_alarm_clusters.len() {
            let cluster = false_alarm_clusters[rng.index(false_alarm_clusters.len())];
            negative_route_km += cluster.surveyed_negative_route_km;
            false_alarms += cluster.false_positives;
        }
        if negative_route_km > 0.0 {
            false_alarm_samples.push(false_alarms as f64 / negative_route_km);
        }
    }

    pod_samples.sort_by(f64::total_cmp);
    false_alarm_samples.sort_by(f64::total_cmp);

    let Some(pod_lower) = percentile(&pod_samples, alpha / 2.0) else {
        metrics.clustered_confidence_interval_status =
            "POD_BOOTSTRAP_UNDEFINED_NOT_RUN".to_string();
        return Ok(metrics);
    };
    let Some(pod_upper) = percentile(&pod_samples, 1.0 - alpha / 2.0) else {
        metrics.clustered_confidence_interval_status =
            "POD_BOOTSTRAP_UNDEFINED_NOT_RUN".to_string();
        return Ok(metrics);
    };
    let Some(far_lower) = percentile(&false_alarm_samples, alpha / 2.0) else {
        metrics.clustered_confidence_interval_status =
            "FALSE_ALARM_BOOTSTRAP_UNDEFINED_NOT_RUN".to_string();
        return Ok(metrics);
    };
    let Some(far_upper) = percentile(&false_alarm_samples, 1.0 - alpha / 2.0) else {
        metrics.clustered_confidence_interval_status =
            "FALSE_ALARM_BOOTSTRAP_UNDEFINED_NOT_RUN".to_string();
        return Ok(metrics);
    };

    metrics.pod_cluster_bootstrap_interval = Some(ClusterConfidenceInterval {
        lower: pod_lower,
        upper: pod_upper,
        confidence_level: config.confidence_level,
        independent_site_clusters: pod_clusters.len(),
        bootstrap_replicates: pod_samples.len(),
    });
    metrics.false_alarm_rate_cluster_bootstrap_interval = Some(ClusterConfidenceInterval {
        lower: far_lower,
        upper: far_upper,
        confidence_level: config.confidence_level,
        independent_site_clusters: false_alarm_clusters.len(),
        bootstrap_replicates: false_alarm_samples.len(),
    });
    metrics.clustered_confidence_interval_status =
        "COMPUTED_SITE_CLUSTER_PERCENTILE_BOOTSTRAP_RESEARCH_ONLY".to_string();
    metrics.qualification_status = "NOT_QUALIFIED_SYNTHETIC_OR_UNQUALIFIED_INPUTS".to_string();

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_envelope() -> SensorObservabilityEnvelope {
        SensorObservabilityEnvelope {
            envelope_id: "soe-gpr-utility-v1".to_string(),
            source_evidence_id: "sensor-qualification-source-1".to_string(),
            sensor_id: "gpr-unit-1".to_string(),
            configuration_id: "antenna-400mhz-v2".to_string(),
            calibration_id: "calibration-2026-11-01".to_string(),
            target_class: "buried_utility".to_string(),
            spatial_support_id: "abstract-route-support".to_string(),
            valid_from_unix_s: 1_700_000_000,
            valid_until_unix_s: 1_800_000_000,
            target_size_m: NumericRange {
                minimum: 0.1,
                maximum: 1.0,
            },
            target_depth_m: NumericRange {
                minimum: 0.2,
                maximum: 2.5,
            },
            target_contrast: NumericRange {
                minimum: 0.1,
                maximum: 1.0,
            },
            environmental_domain: BTreeMap::from([
                (
                    "moisture_fraction".to_string(),
                    NumericRange {
                        minimum: 0.0,
                        maximum: 0.3,
                    },
                ),
                (
                    "conductivity_s_m".to_string(),
                    NumericRange {
                        minimum: 0.001,
                        maximum: 0.2,
                    },
                ),
            ]),
            minimum_pod_requirement: 0.90,
            maximum_false_alarms_per_km: 0.5,
        }
    }

    fn trial(
        id: &str,
        partition: DetectionPartition,
        site: &str,
        acquisition: &str,
        present: bool,
        detected: bool,
        target_index: usize,
    ) -> DetectionTrial {
        let mut environment = BTreeMap::new();
        environment.insert("moisture_fraction".to_string(), 0.12);
        environment.insert("conductivity_s_m".to_string(), 0.05);
        DetectionTrial {
            trial_id: id.to_string(),
            partition,
            target_id: present.then(|| format!("target-{target_index}")),
            site_id: site.to_string(),
            acquisition_id: acquisition.to_string(),
            observed_at_unix_s: 1_750_000_000,
            sensor_id: "gpr-unit-1".to_string(),
            configuration_id: "antenna-400mhz-v2".to_string(),
            calibration_id: "calibration-2026-11-01".to_string(),
            target_class: "buried_utility".to_string(),
            target_present: present,
            detector_detected: detected,
            target_size_m: present.then_some(0.4),
            target_depth_m: present.then_some(1.2),
            target_contrast: present.then_some(0.6),
            environmental_values: environment,
            surveyed_route_km: 0.25,
            localization_error_m: (present && detected).then_some(0.2 + target_index as f64 * 0.01),
        }
    }

    fn fixture() -> SensorObservabilityFixture {
        let mut trials = vec![
            trial(
                "cal-1",
                DetectionPartition::Calibration,
                "cal-site",
                "cal-acq",
                true,
                true,
                1,
            ),
            trial(
                "blind-p1",
                DetectionPartition::BlindTest,
                "blind-site-a",
                "blind-acq-1",
                true,
                true,
                2,
            ),
            trial(
                "blind-p2",
                DetectionPartition::BlindTest,
                "blind-site-a",
                "blind-acq-1",
                true,
                true,
                3,
            ),
            trial(
                "blind-p3",
                DetectionPartition::BlindTest,
                "blind-site-b",
                "blind-acq-2",
                true,
                true,
                4,
            ),
            trial(
                "blind-p4",
                DetectionPartition::BlindTest,
                "blind-site-b",
                "blind-acq-2",
                true,
                true,
                5,
            ),
            trial(
                "blind-p5",
                DetectionPartition::BlindTest,
                "blind-site-c",
                "blind-acq-3",
                true,
                true,
                6,
            ),
            trial(
                "blind-p6",
                DetectionPartition::BlindTest,
                "blind-site-c",
                "blind-acq-3",
                true,
                true,
                7,
            ),
            trial(
                "blind-p7",
                DetectionPartition::BlindTest,
                "blind-site-d",
                "blind-acq-4",
                true,
                true,
                8,
            ),
            trial(
                "blind-p8",
                DetectionPartition::BlindTest,
                "blind-site-d",
                "blind-acq-4",
                true,
                false,
                9,
            ),
        ];
        trials.extend([
            trial(
                "blind-n1",
                DetectionPartition::BlindTest,
                "blind-site-a",
                "blind-acq-1",
                false,
                true,
                10,
            ),
            trial(
                "blind-n2",
                DetectionPartition::BlindTest,
                "blind-site-a",
                "blind-acq-1",
                false,
                false,
                11,
            ),
            trial(
                "blind-n3",
                DetectionPartition::BlindTest,
                "blind-site-b",
                "blind-acq-2",
                false,
                false,
                12,
            ),
            trial(
                "blind-n4",
                DetectionPartition::BlindTest,
                "blind-site-b",
                "blind-acq-2",
                false,
                false,
                13,
            ),
            trial(
                "blind-n5",
                DetectionPartition::BlindTest,
                "blind-site-c",
                "blind-acq-3",
                false,
                false,
                14,
            ),
            trial(
                "blind-n6",
                DetectionPartition::BlindTest,
                "blind-site-d",
                "blind-acq-4",
                false,
                false,
                15,
            ),
        ]);
        let mut out_of_domain = trial(
            "blind-ood-1",
            DetectionPartition::BlindTest,
            "blind-site-e",
            "blind-acq-5",
            true,
            false,
            16,
        );
        out_of_domain
            .environmental_values
            .insert("conductivity_s_m".to_string(), 0.7);
        trials.push(out_of_domain);

        SensorObservabilityFixture {
            benchmark_id: "EARTH-OBSERVABILITY-SENSOR-OBSERVABILITY-001".to_string(),
            fixture_id: "SOE-ANALYTIC-UTILITY-DETECTION-001".to_string(),
            fixture_version: "0.1.0".to_string(),
            seed: 20_261_113,
            status: "synthetic_analytic_detection_trials_not_physical_data".to_string(),
            geography: "abstract_non_geographic".to_string(),
            claim_boundary:
                "Analytic detection scoring fixture only; no sensor physics or field qualification."
                    .to_string(),
            envelope: valid_envelope(),
            trials,
        }
    }

    #[test]
    fn envelope_accepts_valid_target_specific_observability_domain() {
        assert!(validate_envelope(&valid_envelope()).is_empty());
    }

    #[test]
    fn envelope_rejects_invalid_bounds_and_non_probability_pod_requirement() {
        let mut envelope = valid_envelope();
        envelope.target_depth_m = NumericRange {
            minimum: 3.0,
            maximum: 1.0,
        };
        envelope.minimum_pod_requirement = 1.2;
        assert!(
            validate_envelope(&envelope)
                .contains(&EnvelopeValidationError::InvalidRange("target_depth_m"))
        );
        assert!(
            validate_envelope(&envelope).contains(&EnvelopeValidationError::InvalidPodRequirement)
        );
    }

    #[test]
    fn invalid_envelope_is_reported_as_a_contract_error() {
        let mut data = fixture();
        data.envelope.minimum_pod_requirement = 1.5;
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::InvalidEnvelope)
        );
    }

    #[test]
    fn calibration_and_blind_test_sites_must_be_disjoint() {
        let mut data = fixture();
        data.trials[0].site_id = "blind-site-a".to_string();
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::CrossPartitionSiteLeakage)
        );
    }

    #[test]
    fn calibration_and_blind_test_acquisitions_must_be_disjoint() {
        let mut data = fixture();
        data.trials[0].acquisition_id = "blind-acq-1".to_string();
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::CrossPartitionAcquisitionLeakage)
        );
    }

    #[test]
    fn out_of_domain_nondetection_is_counted_separately_not_as_false_negative() {
        let result = evaluate_blind_trials(&fixture()).unwrap();
        assert_eq!(result.positive_targets, 8);
        assert_eq!(result.true_positives, 7);
        assert_eq!(result.false_negatives, 1);
        assert_eq!(result.out_of_domain_blind_trials, 1);
        assert_eq!(result.probability_of_detection, Some(7.0 / 8.0));
    }

    #[test]
    fn false_alarm_rate_uses_only_in_domain_negative_control_route_length() {
        let result = evaluate_blind_trials(&fixture()).unwrap();
        assert_eq!(result.negative_controls, 6);
        assert_eq!(result.false_positives, 1);
        assert_eq!(result.false_alarms_per_km, Some(2.0 / 3.0));
    }

    #[test]
    fn localization_error_is_scored_separately_from_detection() {
        let result = evaluate_blind_trials(&fixture()).unwrap();
        assert_eq!(result.localization_sample_count, 7);
        assert_eq!(result.per_site.len(), 4);
        let site_a = result
            .per_site
            .iter()
            .find(|site| site.site_id == "blind-site-a")
            .unwrap();
        assert_eq!(site_a.positive_targets, 2);
        assert_eq!(site_a.true_positives, 2);
        assert_eq!(site_a.negative_controls, 2);
        assert_eq!(site_a.false_positives, 1);
        assert_eq!(site_a.probability_of_detection, Some(1.0));
        assert_eq!(site_a.false_alarms_per_km, Some(2.0));
        assert!(result.localization_median_error_m.is_some());
        assert!(result.localization_p95_error_m.is_some());
        assert_eq!(
            result.qualification_status,
            "NOT_QUALIFIED_POINT_ESTIMATES_ONLY"
        );
    }

    #[test]
    fn cluster_bootstrap_runs_deterministically_when_preregistered_cluster_minimum_is_met() {
        let config = ClusterBootstrapConfig {
            confidence_level: 0.95,
            bootstrap_replicates: 2_000,
            minimum_independent_site_clusters: 4,
            seed: 20_261_113,
        };
        let first = evaluate_blind_trials_with_cluster_bootstrap(&fixture(), config).unwrap();
        let second = evaluate_blind_trials_with_cluster_bootstrap(&fixture(), config).unwrap();
        assert_eq!(first, second);
        let pod = first.pod_cluster_bootstrap_interval.unwrap();
        let far = first.false_alarm_rate_cluster_bootstrap_interval.unwrap();
        assert!(pod.lower <= pod.upper);
        assert!(far.lower <= far.upper);
        assert_eq!(pod.independent_site_clusters, 4);
        assert_eq!(far.independent_site_clusters, 4);
        assert_eq!(
            first.clustered_confidence_interval_status,
            "COMPUTED_SITE_CLUSTER_PERCENTILE_BOOTSTRAP_RESEARCH_ONLY"
        );
        assert_eq!(
            first.qualification_status,
            "NOT_QUALIFIED_SYNTHETIC_OR_UNQUALIFIED_INPUTS"
        );
    }

    #[test]
    fn cluster_bootstrap_refuses_to_report_intervals_below_preregistered_minimum() {
        let config = ClusterBootstrapConfig {
            confidence_level: 0.95,
            bootstrap_replicates: 2_000,
            minimum_independent_site_clusters: 5,
            seed: 20_261_113,
        };
        let result = evaluate_blind_trials_with_cluster_bootstrap(&fixture(), config).unwrap();
        assert!(result.pod_cluster_bootstrap_interval.is_none());
        assert!(result.false_alarm_rate_cluster_bootstrap_interval.is_none());
        assert_eq!(
            result.clustered_confidence_interval_status,
            "INSUFFICIENT_SITE_CLUSTERS_POD_4_FAR_4_REQUIRED_5_NOT_RUN"
        );
    }

    #[test]
    fn cluster_bootstrap_rejects_invalid_configuration_without_fabricating_intervals() {
        let config = ClusterBootstrapConfig {
            confidence_level: 1.0,
            bootstrap_replicates: 10,
            minimum_independent_site_clusters: 0,
            seed: 20_261_113,
        };
        let result = evaluate_blind_trials_with_cluster_bootstrap(&fixture(), config).unwrap();
        assert!(result.pod_cluster_bootstrap_interval.is_none());
        assert_eq!(
            result.clustered_confidence_interval_status,
            "INVALID_BOOTSTRAP_CONFIGURATION_NOT_RUN"
        );
    }

    #[test]
    fn non_detection_within_envelope_does_not_itself_establish_absence() {
        let result = evaluate_blind_trials(&fixture()).unwrap();
        assert_eq!(result.false_negatives, 1);
        assert_eq!(
            result.clustered_confidence_interval_status,
            "NOT_RUN_CLUSTER_AWARE_BOOTSTRAP_REQUIRED"
        );
    }

    #[test]
    fn target_must_not_be_scored_twice_across_partitions() {
        let mut data = fixture();
        data.trials[1].target_id = Some("target-1".to_string());
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::CrossPartitionTargetLeakage)
        );
    }

    #[test]
    fn missing_environmental_covariate_is_rejected_not_zero_filled() {
        let mut data = fixture();
        data.trials[1]
            .environmental_values
            .remove("moisture_fraction");
        assert!(validate_fixture(&data).unwrap_err().contains(
            &TrialValidationError::MissingEnvironmentalVariable("moisture_fraction".to_string())
        ));
    }

    #[test]
    fn detected_present_target_requires_localization_error() {
        let mut data = fixture();
        data.trials[1].localization_error_m = None;
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::MissingLocalizationError)
        );
    }

    #[test]
    fn absent_control_cannot_carry_target_identity_or_dimensions() {
        let mut data = fixture();
        data.trials[9].target_id = Some("fabricated-target".to_string());
        assert!(
            validate_fixture(&data)
                .unwrap_err()
                .contains(&TrialValidationError::UnexpectedTargetIdentity)
        );
    }
}
