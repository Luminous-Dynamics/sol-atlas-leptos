// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reproducible heritage evaluation split manifests and leakage screening.
//!
//! The audit distinguishes paper-faithful random-tile reproduction from
//! source-footprint, buffered spatial, site, sensor and epoch transfer.
//! It detects declared pixel-window, bounding-box, site, block and physical
//! entity overlaps. It does not prove randomisation, polygon non-intersection,
//! source custody or statistical independence.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_EVALUATION_SCHEMA_VERSION: &str = "luminous.heritage-evaluation-split.v0.1";
pub const MAX_EVALUATION_SAMPLES: usize = 250_000;
pub const MAX_CANDIDATE_PAIR_CHECKS: usize = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationSplitProfile {
    PaperFaithfulRandomTiles,
    SourceFootprintIndependent,
    BufferedSpatialBlocks,
    LeaveOneSiteOut,
    CrossSensorTransfer,
    CrossEpochTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitRole {
    Train,
    Validation,
    Test,
    BufferExcluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialBufferBasis {
    EmpiricalCorrelationRange,
    DeclaredDeploymentDistance,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PixelWindow {
    pub min_x_px: u32,
    pub min_y_px: u32,
    pub max_x_px: u32,
    pub max_y_px: u32,
}

impl PixelWindow {
    fn valid(self) -> bool {
        self.min_x_px < self.max_x_px && self.min_y_px < self.max_y_px
    }

    fn intersects(self, other: Self) -> bool {
        self.min_x_px < other.max_x_px
            && other.min_x_px < self.max_x_px
            && self.min_y_px < other.max_y_px
            && other.min_y_px < self.max_y_px
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetricBounds {
    pub min_x_m: f64,
    pub min_y_m: f64,
    pub max_x_m: f64,
    pub max_y_m: f64,
}

impl MetricBounds {
    fn valid(self) -> bool {
        self.min_x_m.is_finite()
            && self.min_y_m.is_finite()
            && self.max_x_m.is_finite()
            && self.max_y_m.is_finite()
            && self.min_x_m < self.max_x_m
            && self.min_y_m < self.max_y_m
    }

    fn distance(self, other: Self) -> f64 {
        let dx = (other.min_x_m - self.max_x_m)
            .max(self.min_x_m - other.max_x_m)
            .max(0.0);
        let dy = (other.min_y_m - self.max_y_m)
            .max(self.min_y_m - other.max_y_m)
            .max(0.0);
        dx.hypot(dy)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationSample {
    pub sample_id: String,
    pub role: SplitRole,
    pub site_id: String,
    pub region_id: String,
    pub sensor_id: String,
    pub acquisition_epoch_id: String,
    pub block_id: Option<String>,
    pub source_capture_id: String,
    pub source_capture_sha256: String,
    pub source_tile_sha256: String,
    pub source_image_width_px: u32,
    pub source_image_height_px: u32,
    pub source_pixel_window: PixelWindow,
    /// Axis-aligned bounds in the manifest's projected metric CRS.
    pub metric_bounds: MetricBounds,
    /// Stable physical identity IDs; annotations/objects are not assumed unique per tile.
    #[serde(default)]
    pub physical_entity_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusion_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageEvaluationSplitManifest {
    pub schema_version: String,
    pub dataset_sha256: String,
    pub split_protocol_sha256: String,
    pub coordinate_reference_id: String,
    pub profile: EvaluationSplitProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub random_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_paper_split_manifest_sha256: Option<String>,
    #[serde(default)]
    pub held_out_site_ids: Vec<String>,
    #[serde(default)]
    pub held_out_sensor_ids: Vec<String>,
    #[serde(default)]
    pub held_out_epoch_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_buffer_basis: Option<SpatialBufferBasis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_buffer_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_correlation_range_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffer_basis_receipt_ref: Option<String>,
    pub samples: Vec<EvaluationSample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitRoleCounts {
    pub train: usize,
    pub validation: usize,
    pub test: usize,
    pub buffer_excluded: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageEvaluationSplitAuditReport {
    pub schema_version: String,
    pub profile: EvaluationSplitProfile,
    pub sample_count: usize,
    pub role_counts: SplitRoleCounts,
    pub shared_capture_digest_count: usize,
    pub shared_tile_digest_count: usize,
    pub source_pixel_overlap_pair_count: usize,
    pub shared_physical_entity_count: usize,
    pub shared_site_count: usize,
    pub shared_block_count: usize,
    pub held_out_identity_count: usize,
    /// None outside the buffered-spatial estimand, where the distance audit is not run.
    pub spatial_buffer_violation_pair_count: Option<usize>,
    /// reference_manifest_digest_recorded, seeded_protocol_reproduction, protocol_reconstruction, or not_applicable.
    pub paper_split_exactness: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageEvaluationSplitError {
    SchemaVersionMismatch,
    InvalidDatasetDigest,
    InvalidProtocolDigest,
    EmptyCoordinateReference,
    TooManySamples,
    NoActiveSamples,
    MissingTrainOrTestPartition,
    EmptySampleId,
    DuplicateSampleId(String),
    EmptySampleIdentity(String),
    InvalidCaptureDigest(String),
    InvalidTileDigest(String),
    InvalidPixelWindow(String),
    InvalidMetricBounds(String),
    DuplicatePhysicalEntityId(String),
    ExcludedSampleMissingReason(String),
    ActiveSampleHasExclusionReason(String),
    CaptureIdDigestMismatch(String),
    InvalidHeldOutIdentity(String),
    DuplicateHeldOutIdentity(String),
    MissingHeldOutIdentity,
    TestPartitionContainsUnheldIdentity(String),
    MissingBlockId(String),
    HeldOutIdentityInFitPartition(String),
    TestPartitionMissingHeldOutIdentity,
    MissingSpatialBufferConfiguration,
    InvalidSpatialBufferConfiguration,
    BufferSmallerThanCorrelationRange,
    MissingBufferBasisReceipt,
    SharedTileDigestAcrossPartitions(String),
    SharedPhysicalEntityAcrossPartitions(String),
    SourcePixelOverlapAcrossPartitions,
    SharedBlockAcrossPartitions(String),
    SpatialBufferViolation {
        sample_a: String,
        sample_b: String,
        pair_count: usize,
    },
    TooManyCandidatePairChecks,
}

impl fmt::Display for HeritageEvaluationSplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HeritageEvaluationSplitError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn role_counts(samples: &[&EvaluationSample]) -> SplitRoleCounts {
    let mut result = SplitRoleCounts {
        train: 0,
        validation: 0,
        test: 0,
        buffer_excluded: 0,
    };
    for sample in samples {
        match sample.role {
            SplitRole::Train => result.train += 1,
            SplitRole::Validation => result.validation += 1,
            SplitRole::Test => result.test += 1,
            SplitRole::BufferExcluded => result.buffer_excluded += 1,
        }
    }
    result
}

fn shared_identity_count<T: Ord + Clone>(
    values: impl IntoIterator<Item = (T, SplitRole)>,
) -> usize {
    let mut roles = BTreeMap::<T, BTreeSet<SplitRole>>::new();
    for (value, role) in values {
        if role != SplitRole::BufferExcluded {
            roles.entry(value).or_default().insert(role);
        }
    }
    roles.values().filter(|set| set.len() > 1).count()
}

fn first_shared_identity<T: Ord + Clone>(
    values: impl IntoIterator<Item = (T, SplitRole)>,
) -> Option<T> {
    let mut roles = BTreeMap::<T, BTreeSet<SplitRole>>::new();
    for (value, role) in values {
        if role != SplitRole::BufferExcluded {
            roles.entry(value).or_default().insert(role);
        }
    }
    roles
        .into_iter()
        .find_map(|(value, split_roles)| (split_roles.len() > 1).then_some(value))
}

fn count_source_pixel_overlaps(
    samples: &[&EvaluationSample],
) -> Result<usize, HeritageEvaluationSplitError> {
    let mut captures = BTreeMap::<&str, Vec<&EvaluationSample>>::new();
    for sample in samples
        .iter()
        .copied()
        .filter(|s| s.role != SplitRole::BufferExcluded)
    {
        captures
            .entry(sample.source_capture_sha256.as_str())
            .or_default()
            .push(sample);
    }

    let mut overlap_count = 0usize;
    let mut candidate_checks = 0usize;
    for group in captures.values_mut() {
        group.sort_by(|a, b| {
            a.source_pixel_window
                .min_x_px
                .cmp(&b.source_pixel_window.min_x_px)
                .then_with(|| a.sample_id.cmp(&b.sample_id))
        });
        let mut active = Vec::<&EvaluationSample>::new();
        for current in group.iter().copied() {
            active.retain(|prior| {
                prior.source_pixel_window.max_x_px > current.source_pixel_window.min_x_px
            });
            for prior in &active {
                if prior.role == current.role {
                    continue;
                }
                candidate_checks += 1;
                if candidate_checks > MAX_CANDIDATE_PAIR_CHECKS {
                    return Err(HeritageEvaluationSplitError::TooManyCandidatePairChecks);
                }
                if prior
                    .source_pixel_window
                    .intersects(current.source_pixel_window)
                {
                    overlap_count += 1;
                }
            }
            active.push(current);
        }
    }
    Ok(overlap_count)
}

fn count_spatial_buffer_violations(
    samples: &[&EvaluationSample],
    buffer_m: f64,
) -> Result<(usize, Option<(String, String)>), HeritageEvaluationSplitError> {
    let mut sorted: Vec<&EvaluationSample> = samples
        .iter()
        .copied()
        .filter(|s| s.role != SplitRole::BufferExcluded)
        .collect();
    sorted.sort_by(|a, b| {
        a.metric_bounds
            .min_x_m
            .total_cmp(&b.metric_bounds.min_x_m)
            .then_with(|| a.sample_id.cmp(&b.sample_id))
    });

    let mut active = Vec::<&EvaluationSample>::new();
    let mut violations = 0usize;
    let mut first_violation = None;
    let mut candidate_checks = 0usize;
    for current in sorted {
        active.retain(|prior| {
            prior.metric_bounds.max_x_m + buffer_m >= current.metric_bounds.min_x_m
        });
        for prior in &active {
            if prior.role == current.role {
                continue;
            }
            candidate_checks += 1;
            if candidate_checks > MAX_CANDIDATE_PAIR_CHECKS {
                return Err(HeritageEvaluationSplitError::TooManyCandidatePairChecks);
            }
            if prior.metric_bounds.distance(current.metric_bounds) + 1e-9 < buffer_m {
                violations += 1;
                if first_violation.is_none() {
                    first_violation = Some((prior.sample_id.clone(), current.sample_id.clone()));
                }
            }
        }
        active.push(current);
    }
    Ok((violations, first_violation))
}

fn validate_held_out_partition(
    held_out: &[String],
    samples: &[&EvaluationSample],
    identity: impl Fn(&EvaluationSample) -> &str,
) -> Result<usize, HeritageEvaluationSplitError> {
    if held_out.is_empty() {
        return Err(HeritageEvaluationSplitError::MissingHeldOutIdentity);
    }
    let mut held = BTreeSet::new();
    for id in held_out {
        if !non_empty(id) {
            return Err(HeritageEvaluationSplitError::InvalidHeldOutIdentity(
                id.clone(),
            ));
        }
        if !held.insert(id.as_str()) {
            return Err(HeritageEvaluationSplitError::DuplicateHeldOutIdentity(
                id.clone(),
            ));
        }
    }
    let mut found_in_test = BTreeSet::new();
    for sample in samples
        .iter()
        .copied()
        .filter(|s| s.role != SplitRole::BufferExcluded)
    {
        let value = identity(sample);
        match sample.role {
            SplitRole::Train | SplitRole::Validation if held.contains(value) => {
                return Err(HeritageEvaluationSplitError::HeldOutIdentityInFitPartition(
                    value.to_owned(),
                ));
            }
            SplitRole::Test if !held.contains(value) => {
                return Err(
                    HeritageEvaluationSplitError::TestPartitionContainsUnheldIdentity(
                        value.to_owned(),
                    ),
                );
            }
            SplitRole::Test => {
                found_in_test.insert(value);
            }
            SplitRole::Train | SplitRole::Validation | SplitRole::BufferExcluded => {}
        }
    }
    if held.iter().any(|id| !found_in_test.contains(id)) {
        return Err(HeritageEvaluationSplitError::TestPartitionMissingHeldOutIdentity);
    }
    Ok(found_in_test.len())
}

pub fn audit_heritage_evaluation_split(
    manifest: &HeritageEvaluationSplitManifest,
) -> Result<HeritageEvaluationSplitAuditReport, HeritageEvaluationSplitError> {
    if manifest.schema_version != HERITAGE_EVALUATION_SCHEMA_VERSION {
        return Err(HeritageEvaluationSplitError::SchemaVersionMismatch);
    }
    if !valid_sha256(&manifest.dataset_sha256) {
        return Err(HeritageEvaluationSplitError::InvalidDatasetDigest);
    }
    if !valid_sha256(&manifest.split_protocol_sha256) {
        return Err(HeritageEvaluationSplitError::InvalidProtocolDigest);
    }
    if manifest
        .reference_paper_split_manifest_sha256
        .as_deref()
        .is_some_and(|digest| !valid_sha256(digest))
    {
        return Err(HeritageEvaluationSplitError::InvalidProtocolDigest);
    }
    if !non_empty(&manifest.coordinate_reference_id) {
        return Err(HeritageEvaluationSplitError::EmptyCoordinateReference);
    }
    if manifest.samples.len() > MAX_EVALUATION_SAMPLES {
        return Err(HeritageEvaluationSplitError::TooManySamples);
    }

    let mut sample_ids = BTreeSet::new();
    let mut capture_digests_by_id = BTreeMap::<String, String>::new();
    for sample in &manifest.samples {
        if !non_empty(&sample.sample_id) {
            return Err(HeritageEvaluationSplitError::EmptySampleId);
        }
        if !sample_ids.insert(sample.sample_id.as_str()) {
            return Err(HeritageEvaluationSplitError::DuplicateSampleId(
                sample.sample_id.clone(),
            ));
        }
        if !non_empty(&sample.site_id)
            || !non_empty(&sample.region_id)
            || !non_empty(&sample.sensor_id)
            || !non_empty(&sample.acquisition_epoch_id)
            || !non_empty(&sample.source_capture_id)
        {
            return Err(HeritageEvaluationSplitError::EmptySampleIdentity(
                sample.sample_id.clone(),
            ));
        }
        if !valid_sha256(&sample.source_capture_sha256) {
            return Err(HeritageEvaluationSplitError::InvalidCaptureDigest(
                sample.sample_id.clone(),
            ));
        }
        if !valid_sha256(&sample.source_tile_sha256) {
            return Err(HeritageEvaluationSplitError::InvalidTileDigest(
                sample.sample_id.clone(),
            ));
        }
        if sample.source_image_width_px == 0
            || sample.source_image_height_px == 0
            || !sample.source_pixel_window.valid()
            || sample.source_pixel_window.max_x_px > sample.source_image_width_px
            || sample.source_pixel_window.max_y_px > sample.source_image_height_px
        {
            return Err(HeritageEvaluationSplitError::InvalidPixelWindow(
                sample.sample_id.clone(),
            ));
        }
        if !sample.metric_bounds.valid() {
            return Err(HeritageEvaluationSplitError::InvalidMetricBounds(
                sample.sample_id.clone(),
            ));
        }
        if sample.role == SplitRole::BufferExcluded {
            if sample
                .exclusion_reason
                .as_deref()
                .is_none_or(|reason| !non_empty(reason))
            {
                return Err(HeritageEvaluationSplitError::ExcludedSampleMissingReason(
                    sample.sample_id.clone(),
                ));
            }
        } else if sample.exclusion_reason.is_some() {
            return Err(
                HeritageEvaluationSplitError::ActiveSampleHasExclusionReason(
                    sample.sample_id.clone(),
                ),
            );
        }
        if capture_digests_by_id
            .get(&sample.source_capture_id)
            .is_some_and(|digest| digest != &sample.source_capture_sha256)
        {
            return Err(HeritageEvaluationSplitError::CaptureIdDigestMismatch(
                sample.source_capture_id.clone(),
            ));
        }
        capture_digests_by_id
            .entry(sample.source_capture_id.clone())
            .or_insert_with(|| sample.source_capture_sha256.clone());
        let mut identities = BTreeSet::new();
        for identity in &sample.physical_entity_ids {
            if !non_empty(identity) || !identities.insert(identity.as_str()) {
                return Err(HeritageEvaluationSplitError::DuplicatePhysicalEntityId(
                    sample.sample_id.clone(),
                ));
            }
        }
    }

    let active: Vec<&EvaluationSample> = manifest
        .samples
        .iter()
        .filter(|s| s.role != SplitRole::BufferExcluded)
        .collect();
    if active.is_empty() {
        return Err(HeritageEvaluationSplitError::NoActiveSamples);
    }
    let counts = role_counts(&manifest.samples.iter().collect::<Vec<_>>());
    if counts.train == 0 || counts.test == 0 {
        return Err(HeritageEvaluationSplitError::MissingTrainOrTestPartition);
    }

    let shared_capture_digest_count = shared_identity_count(
        active
            .iter()
            .map(|sample| (sample.source_capture_sha256.clone(), sample.role)),
    );
    let shared_tile_digest_count = shared_identity_count(
        active
            .iter()
            .map(|sample| (sample.source_tile_sha256.clone(), sample.role)),
    );
    let shared_site_count = shared_identity_count(
        active
            .iter()
            .map(|sample| (sample.site_id.clone(), sample.role)),
    );
    let shared_block_count = shared_identity_count(
        active
            .iter()
            .filter_map(|sample| sample.block_id.clone().map(|id| (id, sample.role))),
    );
    let shared_physical_entity_count = shared_identity_count(active.iter().flat_map(|sample| {
        sample
            .physical_entity_ids
            .iter()
            .cloned()
            .map(move |id| (id, sample.role))
    }));
    let source_pixel_overlap_pair_count = count_source_pixel_overlaps(&active)?;

    let mut held_out_identity_count = 0usize;
    match manifest.profile {
        EvaluationSplitProfile::PaperFaithfulRandomTiles => {}
        EvaluationSplitProfile::SourceFootprintIndependent => {
            if source_pixel_overlap_pair_count > 0 {
                return Err(HeritageEvaluationSplitError::SourcePixelOverlapAcrossPartitions);
            }
            if let Some(tile_id) = first_shared_identity(active.iter().flat_map(|sample| {
                std::iter::once((sample.source_tile_sha256.clone(), sample.role))
            })) {
                return Err(
                    HeritageEvaluationSplitError::SharedTileDigestAcrossPartitions(tile_id),
                );
            }
            if let Some(entity_id) = first_shared_identity(active.iter().flat_map(|sample| {
                sample
                    .physical_entity_ids
                    .iter()
                    .cloned()
                    .map(move |id| (id, sample.role))
            })) {
                return Err(
                    HeritageEvaluationSplitError::SharedPhysicalEntityAcrossPartitions(entity_id),
                );
            }
        }
        EvaluationSplitProfile::BufferedSpatialBlocks => {
            if source_pixel_overlap_pair_count > 0 {
                return Err(HeritageEvaluationSplitError::SourcePixelOverlapAcrossPartitions);
            }
            if shared_tile_digest_count > 0 {
                let id = first_shared_identity(
                    active
                        .iter()
                        .map(|sample| (sample.source_tile_sha256.clone(), sample.role)),
                )
                .unwrap_or_default();
                return Err(HeritageEvaluationSplitError::SharedTileDigestAcrossPartitions(id));
            }
            if let Some(entity_id) = first_shared_identity(active.iter().flat_map(|sample| {
                sample
                    .physical_entity_ids
                    .iter()
                    .cloned()
                    .map(move |id| (id, sample.role))
            })) {
                return Err(
                    HeritageEvaluationSplitError::SharedPhysicalEntityAcrossPartitions(entity_id),
                );
            }
            if active
                .iter()
                .any(|sample| sample.block_id.as_deref().is_none_or(|id| !non_empty(id)))
            {
                let sample_id = active
                    .iter()
                    .find(|sample| sample.block_id.as_deref().is_none_or(|id| !non_empty(id)))
                    .map(|sample| sample.sample_id.clone())
                    .unwrap_or_default();
                return Err(HeritageEvaluationSplitError::MissingBlockId(sample_id));
            }
            if shared_block_count > 0 {
                return Err(HeritageEvaluationSplitError::SharedBlockAcrossPartitions(
                    "block ID appears in multiple roles".to_owned(),
                ));
            }
            let basis = manifest
                .spatial_buffer_basis
                .ok_or(HeritageEvaluationSplitError::MissingSpatialBufferConfiguration)?;
            let buffer = manifest
                .spatial_buffer_m
                .ok_or(HeritageEvaluationSplitError::MissingSpatialBufferConfiguration)?;
            if !buffer.is_finite() || buffer <= 0.0 {
                return Err(HeritageEvaluationSplitError::InvalidSpatialBufferConfiguration);
            }
            match basis {
                SpatialBufferBasis::EmpiricalCorrelationRange => {
                    let range = manifest
                        .spatial_correlation_range_m
                        .ok_or(HeritageEvaluationSplitError::MissingSpatialBufferConfiguration)?;
                    if !range.is_finite() || range <= 0.0 {
                        return Err(
                            HeritageEvaluationSplitError::InvalidSpatialBufferConfiguration,
                        );
                    }
                    if buffer < range {
                        return Err(
                            HeritageEvaluationSplitError::BufferSmallerThanCorrelationRange,
                        );
                    }
                }
                SpatialBufferBasis::DeclaredDeploymentDistance => {}
            }
            if manifest
                .buffer_basis_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
            {
                return Err(HeritageEvaluationSplitError::MissingBufferBasisReceipt);
            }
            let (violations, first) = count_spatial_buffer_violations(&active, buffer)?;
            if violations > 0 {
                let (sample_a, sample_b) = first.expect("nonzero violation has a pair");
                return Err(HeritageEvaluationSplitError::SpatialBufferViolation {
                    sample_a,
                    sample_b,
                    pair_count: violations,
                });
            }
        }
        EvaluationSplitProfile::LeaveOneSiteOut => {
            held_out_identity_count =
                validate_held_out_partition(&manifest.held_out_site_ids, &active, |sample| {
                    sample.site_id.as_str()
                })?;
            if source_pixel_overlap_pair_count > 0 {
                return Err(HeritageEvaluationSplitError::SourcePixelOverlapAcrossPartitions);
            }
            if let Some(tile_id) = first_shared_identity(
                active
                    .iter()
                    .map(|sample| (sample.source_tile_sha256.clone(), sample.role)),
            ) {
                return Err(
                    HeritageEvaluationSplitError::SharedTileDigestAcrossPartitions(tile_id),
                );
            }
            if let Some(entity_id) = first_shared_identity(active.iter().flat_map(|sample| {
                sample
                    .physical_entity_ids
                    .iter()
                    .cloned()
                    .map(move |id| (id, sample.role))
            })) {
                return Err(
                    HeritageEvaluationSplitError::SharedPhysicalEntityAcrossPartitions(
                        "duplicate physical entity".to_owned(),
                    ),
                );
            }
        }
        EvaluationSplitProfile::CrossSensorTransfer => {
            held_out_identity_count =
                validate_held_out_partition(&manifest.held_out_sensor_ids, &active, |sample| {
                    sample.sensor_id.as_str()
                })?;
        }
        EvaluationSplitProfile::CrossEpochTransfer => {
            held_out_identity_count =
                validate_held_out_partition(&manifest.held_out_epoch_ids, &active, |sample| {
                    sample.acquisition_epoch_id.as_str()
                })?;
        }
    }

    let claim_ceiling = match manifest.profile {
        EvaluationSplitProfile::PaperFaithfulRandomTiles => "paper_split_reproduction_only",
        EvaluationSplitProfile::SourceFootprintIndependent => {
            "source_pixel_and_physical_entity_independent_partition_only"
        }
        EvaluationSplitProfile::BufferedSpatialBlocks => {
            "buffered_spatial_transfer_axis_aligned_bounds_screening_only"
        }
        EvaluationSplitProfile::LeaveOneSiteOut => "leave_one_site_out_partition_only",
        EvaluationSplitProfile::CrossSensorTransfer => "held_out_sensor_partition_only",
        EvaluationSplitProfile::CrossEpochTransfer => "held_out_epoch_partition_only",
    };

    let paper_split_exactness = match (
        manifest.profile,
        manifest.reference_paper_split_manifest_sha256.is_some(),
        manifest.random_seed.is_some(),
    ) {
        (EvaluationSplitProfile::PaperFaithfulRandomTiles, true, _) => {
            "reference_manifest_digest_recorded"
        }
        (EvaluationSplitProfile::PaperFaithfulRandomTiles, false, true) => {
            "seeded_protocol_reproduction"
        }
        (EvaluationSplitProfile::PaperFaithfulRandomTiles, false, false) => {
            "protocol_reconstruction"
        }
        _ => "not_applicable",
    };

    Ok(HeritageEvaluationSplitAuditReport {
        schema_version: HERITAGE_EVALUATION_SCHEMA_VERSION.to_owned(),
        profile: manifest.profile,
        sample_count: manifest.samples.len(),
        role_counts: counts,
        shared_capture_digest_count,
        shared_tile_digest_count,
        source_pixel_overlap_pair_count,
        shared_physical_entity_count,
        shared_site_count,
        shared_block_count,
        spatial_buffer_violation_pair_count: if manifest.profile
            == EvaluationSplitProfile::BufferedSpatialBlocks
        {
            Some(0)
        } else {
            None
        },
        held_out_identity_count,
        paper_split_exactness: paper_split_exactness.to_owned(),
        claim_ceiling: claim_ceiling.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(char: char) -> String {
        std::iter::repeat_n(char, 64).collect()
    }

    fn sample(
        id: &str,
        role: SplitRole,
        site: &str,
        capture: &str,
        tile: &str,
        window: PixelWindow,
        bounds: MetricBounds,
        feature_ids: &[&str],
    ) -> EvaluationSample {
        EvaluationSample {
            sample_id: id.to_owned(),
            role,
            site_id: site.to_owned(),
            region_id: "region-1".to_owned(),
            sensor_id: "drone-camera-1".to_owned(),
            acquisition_epoch_id: "epoch-1".to_owned(),
            block_id: Some(format!("block-{id}")),
            source_capture_id: capture.to_owned(),
            source_capture_sha256: digest('a'),
            source_tile_sha256: digest(tile.chars().last().unwrap_or('b')),
            source_image_width_px: 1000,
            source_image_height_px: 1000,
            source_pixel_window: window,
            metric_bounds: bounds,
            physical_entity_ids: feature_ids.iter().map(|s| (*s).to_owned()).collect(),
            exclusion_reason: None,
        }
    }

    fn base_manifest(profile: EvaluationSplitProfile) -> HeritageEvaluationSplitManifest {
        HeritageEvaluationSplitManifest {
            schema_version: HERITAGE_EVALUATION_SCHEMA_VERSION.to_owned(),
            dataset_sha256: digest('c'),
            split_protocol_sha256: digest('d'),
            coordinate_reference_id: "EPSG:32735".to_owned(),
            profile,
            random_seed: Some(42),
            reference_paper_split_manifest_sha256: None,
            held_out_site_ids: Vec::new(),
            held_out_sensor_ids: Vec::new(),
            held_out_epoch_ids: Vec::new(),
            spatial_buffer_basis: None,
            spatial_buffer_m: None,
            spatial_correlation_range_m: None,
            buffer_basis_receipt_ref: None,
            samples: vec![
                sample(
                    "train-1",
                    SplitRole::Train,
                    "site-1",
                    "capture-1",
                    "tile-1",
                    PixelWindow {
                        min_x_px: 0,
                        min_y_px: 0,
                        max_x_px: 100,
                        max_y_px: 100,
                    },
                    MetricBounds {
                        min_x_m: 0.0,
                        min_y_m: 0.0,
                        max_x_m: 10.0,
                        max_y_m: 10.0,
                    },
                    &["stone-1"],
                ),
                sample(
                    "test-1",
                    SplitRole::Test,
                    "site-1",
                    "capture-1",
                    "tile-2",
                    PixelWindow {
                        min_x_px: 200,
                        min_y_px: 0,
                        max_x_px: 300,
                        max_y_px: 100,
                    },
                    MetricBounds {
                        min_x_m: 20.0,
                        min_y_m: 0.0,
                        max_x_m: 30.0,
                        max_y_m: 10.0,
                    },
                    &["stone-2"],
                ),
            ],
        }
    }

    #[test]
    fn paper_faithful_random_tiles_report_overlap_without_claiming_transfer() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.samples[1].source_pixel_window = PixelWindow {
            min_x_px: 50,
            min_y_px: 0,
            max_x_px: 150,
            max_y_px: 100,
        };
        input.samples[1].physical_entity_ids = vec!["stone-1".to_owned()];
        let report = audit_heritage_evaluation_split(&input).expect("paper split is auditable");
        assert_eq!(report.source_pixel_overlap_pair_count, 1);
        assert_eq!(report.shared_physical_entity_count, 1);
        assert_eq!(report.claim_ceiling, "paper_split_reproduction_only");
    }

    #[test]
    fn source_footprint_profile_rejects_overlapping_source_pixels() {
        let mut input = base_manifest(EvaluationSplitProfile::SourceFootprintIndependent);
        input.samples[1].source_pixel_window = PixelWindow {
            min_x_px: 50,
            min_y_px: 0,
            max_x_px: 150,
            max_y_px: 100,
        };
        assert_eq!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::SourcePixelOverlapAcrossPartitions)
        );
    }

    #[test]
    fn source_footprint_profile_rejects_shared_physical_identity() {
        let mut input = base_manifest(EvaluationSplitProfile::SourceFootprintIndependent);
        input.samples[1].physical_entity_ids = vec!["stone-1".to_owned()];
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::SharedPhysicalEntityAcrossPartitions(_))
        ));
    }

    #[test]
    fn paper_reference_digest_is_recorded_without_claiming_exact_authentication() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.random_seed = None;
        input.reference_paper_split_manifest_sha256 = Some(digest('e'));
        let report = audit_heritage_evaluation_split(&input).expect("reference digest recorded");
        assert_eq!(
            report.paper_split_exactness,
            "reference_manifest_digest_recorded"
        );
    }

    #[test]
    fn paper_protocol_without_seed_is_labelled_protocol_reconstruction() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.random_seed = None;
        let report = audit_heritage_evaluation_split(&input).expect("paper protocol record");
        assert_eq!(report.paper_split_exactness, "protocol_reconstruction");
    }

    #[test]
    fn strict_profile_rejects_duplicate_tile_digest_without_window_overlap() {
        let mut input = base_manifest(EvaluationSplitProfile::SourceFootprintIndependent);
        input.samples[1].source_tile_sha256 = input.samples[0].source_tile_sha256.clone();
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::SharedTileDigestAcrossPartitions(_))
        ));
    }

    #[test]
    fn cross_sensor_profile_rejects_held_out_sensor_in_training() {
        let mut input = base_manifest(EvaluationSplitProfile::CrossSensorTransfer);
        input.held_out_sensor_ids = vec!["drone-camera-1".to_owned()];
        input.samples[1].sensor_id = "lidar-1".to_owned();
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::HeldOutIdentityInFitPartition(
                _
            ))
        ));
    }

    #[test]
    fn buffered_spatial_error_reports_the_violating_pair_count() {
        let mut input = base_manifest(EvaluationSplitProfile::BufferedSpatialBlocks);
        input.samples[1].block_id = Some("block-test".to_owned());
        input.samples[1].metric_bounds = MetricBounds {
            min_x_m: 12.0,
            min_y_m: 0.0,
            max_x_m: 22.0,
            max_y_m: 10.0,
        };
        input.spatial_buffer_basis = Some(SpatialBufferBasis::DeclaredDeploymentDistance);
        input.spatial_buffer_m = Some(5.0);
        input.buffer_basis_receipt_ref = Some("deployment-distance://receipt/1".to_owned());
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::SpatialBufferViolation { pair_count: 1, .. })
        ));
    }

    #[test]
    fn source_footprint_profile_accepts_disjoint_tiles_from_same_capture() {
        let input = base_manifest(EvaluationSplitProfile::SourceFootprintIndependent);
        let report = audit_heritage_evaluation_split(&input).expect("disjoint tiles");
        assert_eq!(report.source_pixel_overlap_pair_count, 0);
        assert_eq!(report.shared_capture_digest_count, 1);
        assert_eq!(report.shared_site_count, 1);
    }

    #[test]
    fn leave_one_site_out_rejects_held_out_site_in_training() {
        let mut input = base_manifest(EvaluationSplitProfile::LeaveOneSiteOut);
        input.held_out_site_ids = vec!["site-1".to_owned()];
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::HeldOutIdentityInFitPartition(
                _
            ))
        ));
    }

    #[test]
    fn leave_one_site_out_accepts_a_wholly_held_out_site() {
        let mut input = base_manifest(EvaluationSplitProfile::LeaveOneSiteOut);
        input.held_out_site_ids = vec!["site-2".to_owned()];
        input.samples[1].site_id = "site-2".to_owned();
        input.samples[1].source_capture_id = "capture-2".to_owned();
        input.samples[1].source_capture_sha256 = digest('e');
        input.samples[1].source_pixel_window = PixelWindow {
            min_x_px: 0,
            min_y_px: 0,
            max_x_px: 100,
            max_y_px: 100,
        };
        input.samples[1].physical_entity_ids = vec!["stone-99".to_owned()];
        let report = audit_heritage_evaluation_split(&input).expect("held out site");
        assert_eq!(report.held_out_identity_count, 1);
        assert_eq!(report.claim_ceiling, "leave_one_site_out_partition_only");
    }

    #[test]
    fn spatial_block_profile_requires_buffer_at_least_correlation_range() {
        let mut input = base_manifest(EvaluationSplitProfile::BufferedSpatialBlocks);
        input.samples[1].block_id = Some("block-test".to_owned());
        input.spatial_buffer_basis = Some(SpatialBufferBasis::EmpiricalCorrelationRange);
        input.spatial_buffer_m = Some(20.0);
        input.spatial_correlation_range_m = Some(25.0);
        input.buffer_basis_receipt_ref = Some("variogram://receipt/1".to_owned());
        assert_eq!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::BufferSmallerThanCorrelationRange)
        );
    }

    #[test]
    fn spatial_block_profile_rejects_bbox_distance_below_buffer() {
        let mut input = base_manifest(EvaluationSplitProfile::BufferedSpatialBlocks);
        input.samples[1].block_id = Some("block-test".to_owned());
        input.samples[1].metric_bounds = MetricBounds {
            min_x_m: 12.0,
            min_y_m: 0.0,
            max_x_m: 22.0,
            max_y_m: 10.0,
        };
        input.spatial_buffer_basis = Some(SpatialBufferBasis::DeclaredDeploymentDistance);
        input.spatial_buffer_m = Some(5.0);
        input.buffer_basis_receipt_ref = Some("deployment-distance://receipt/1".to_owned());
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::SpatialBufferViolation { .. })
        ));
    }

    #[test]
    fn buffer_excluded_samples_remain_in_manifest_with_reasons() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        let mut excluded = input.samples[1].clone();
        excluded.sample_id = "buffer-1".to_owned();
        excluded.role = SplitRole::BufferExcluded;
        excluded.exclusion_reason =
            Some("inside preregistered spatial exclusion buffer".to_owned());
        input.samples.push(excluded);
        let report = audit_heritage_evaluation_split(&input).expect("auditable exclusion");
        assert_eq!(report.role_counts.buffer_excluded, 1);
        assert_eq!(report.sample_count, 3);
    }

    #[test]
    fn excluded_sample_without_reason_is_rejected() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.samples[1].role = SplitRole::BufferExcluded;
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::ExcludedSampleMissingReason(_))
        ));
    }

    #[test]
    fn cross_sensor_profile_requires_sensor_holdout() {
        let mut input = base_manifest(EvaluationSplitProfile::CrossSensorTransfer);
        input.held_out_sensor_ids = vec!["lidar-1".to_owned()];
        input.samples[1].sensor_id = "lidar-1".to_owned();
        let report = audit_heritage_evaluation_split(&input).expect("sensor holdout");
        assert_eq!(report.held_out_identity_count, 1);
    }

    #[test]
    fn cross_epoch_profile_requires_epoch_holdout() {
        let mut input = base_manifest(EvaluationSplitProfile::CrossEpochTransfer);
        input.held_out_epoch_ids = vec!["epoch-2".to_owned()];
        input.samples[1].acquisition_epoch_id = "epoch-2".to_owned();
        let report = audit_heritage_evaluation_split(&input).expect("epoch holdout");
        assert_eq!(report.held_out_identity_count, 1);
    }

    #[test]
    fn capture_id_cannot_map_to_multiple_capture_digests() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.samples[1].source_capture_sha256 = digest('f');
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::CaptureIdDigestMismatch(_))
        ));
    }

    #[test]
    fn duplicate_sample_identity_and_invalid_bounds_are_rejected() {
        let mut input = base_manifest(EvaluationSplitProfile::PaperFaithfulRandomTiles);
        input.samples[1].sample_id = input.samples[0].sample_id.clone();
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::DuplicateSampleId(_))
        ));
        input.samples[1].sample_id = "test-1".to_owned();
        input.samples[1].metric_bounds.max_x_m = input.samples[1].metric_bounds.min_x_m;
        assert!(matches!(
            audit_heritage_evaluation_split(&input),
            Err(HeritageEvaluationSplitError::InvalidMetricBounds(_))
        ));
    }

    #[test]
    fn split_schemas_and_fixture_are_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-evaluation-split-v0.1.schema.json"
        ))
        .expect("schema JSON");
        let fixture: HeritageEvaluationSplitManifest = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-evaluation-paper-split-synthetic-v0.1.json"
        ))
        .expect("fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        let report = audit_heritage_evaluation_split(&fixture).expect("paper fixture audit");
        assert_eq!(report.source_pixel_overlap_pair_count, 1);
        assert_eq!(report.shared_physical_entity_count, 2);
    }
}
