// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Leakage guards for site-held-out and spatial-block Heritage Discovery
//! Benchmark manifests.
//!
//! Site-level disjointness is checked directly. Spatial-block distances are
//! supplied by a separately versioned geospatial audit; this module validates
//! the declared distance/buffer contract but does not calculate geometry
//! distances from raster or vector assets.

use crate::heritage::{parse_rfc3339_nanos, sha256_hex};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkSplitProtocol {
    HeldOutSite,
    HeldOutRegion,
    LeaveOneBiomeOut,
    LeaveOneSensorOut,
    TemporalHoldout,
    ExternalBlind,
    WithinSiteSpatialBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkPartition {
    Train,
    Validation,
    Test,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialBlockAssignment {
    pub site_id: String,
    pub block_id: String,
    pub partition: BenchmarkPartition,
    /// Distance to the nearest block in another partition, computed by an
    /// external CRS-aware geometry audit. This module checks the declared
    /// value against the preregistered buffer; it does not recompute it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nearest_other_partition_distance_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkSampleAssignment {
    pub sample_id: String,
    pub site_id: String,
    pub partition: BenchmarkPartition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biome_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_end_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkSplitManifest {
    pub schema_version: String,
    pub split_id: String,
    pub protocol: BenchmarkSplitProtocol,
    /// For site/region/sensor/time holdouts, site membership must be disjoint.
    /// For within-site spatial blocks, membership is derived from block assignments.
    #[serde(default)]
    pub train_site_ids: Vec<String>,
    #[serde(default)]
    pub validation_site_ids: Vec<String>,
    #[serde(default)]
    pub test_site_ids: Vec<String>,
    #[serde(default)]
    pub spatial_block_assignments: Vec<SpatialBlockAssignment>,
    #[serde(default)]
    pub sample_assignments: Vec<BenchmarkSampleAssignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_out_region_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_out_biome_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_out_sensor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal_cutoff_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_blind_evaluation_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_spatial_dependence_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_buffer_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_distance_audit_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitValidationScope {
    SiteIdDisjointness,
    DeclaredExternalSpatialBlockAudit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitValidationReceipt {
    pub schema_version: String,
    pub split_id: String,
    pub protocol: BenchmarkSplitProtocol,
    pub scope: SplitValidationScope,
    pub train_site_count: usize,
    pub validation_site_count: usize,
    pub test_site_count: usize,
    pub spatial_block_count: usize,
    pub sample_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_spatial_dependence_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_spatial_buffer_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_declared_cross_partition_distance_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_distance_audit_ref: Option<String>,
    /// False by design: an external audit reference and supplied distances
    /// are checked for completeness/thresholds, not geometrically recomputed here.
    pub geometry_distances_recomputed_by_core: bool,
    pub manifest_sha256: String,
    pub receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitValidationError {
    UnsupportedSchemaVersion,
    EmptySplitId,
    EmptySiteId,
    DuplicateSiteIdWithinPartition {
        partition: BenchmarkPartition,
        site_id: String,
    },
    SiteOverlapsPartitions {
        site_id: String,
    },
    EmptySitePartition(BenchmarkPartition),
    MissingSpatialDependenceEstimate,
    InvalidSpatialDependenceEstimate,
    MissingSpatialBuffer,
    InvalidSpatialBuffer,
    SpatialBufferBelowDependence {
        buffer_m: f64,
        estimated_dependence_m: f64,
    },
    MissingSpatialDistanceAuditReference,
    EmptySpatialBlockAssignments,
    EmptySpatialBlockSiteId,
    EmptySpatialBlockId,
    DuplicateSpatialBlock {
        site_id: String,
        block_id: String,
    },
    MissingCrossPartitionDistance {
        site_id: String,
        block_id: String,
    },
    InvalidCrossPartitionDistance {
        site_id: String,
        block_id: String,
    },
    CrossPartitionDistanceBelowBuffer {
        site_id: String,
        block_id: String,
        distance_m: f64,
        buffer_m: f64,
    },
    MissingSpatialBlockPartition(BenchmarkPartition),
    SpatialBlockSiteListMismatch(BenchmarkPartition),
    MissingSampleAssignments,
    EmptySampleId,
    DuplicateSampleId(String),
    MissingRegionId {
        sample_id: String,
    },
    MissingBiomeId {
        sample_id: String,
    },
    MissingSensorId {
        sample_id: String,
    },
    MissingAcquisitionInterval {
        sample_id: String,
    },
    InvalidSampleAcquisitionInterval {
        sample_id: String,
    },
    MissingHeldOutRegionId,
    HeldOutRegionMismatch {
        sample_id: String,
        expected: String,
    },
    MissingHeldOutBiomeId,
    HeldOutBiomeMismatch {
        sample_id: String,
        expected: String,
    },
    MissingHeldOutSensorId,
    HeldOutSensorMismatch {
        sample_id: String,
        expected: String,
    },
    MissingTemporalCutoff,
    TemporalPartitionViolation {
        sample_id: String,
        partition: BenchmarkPartition,
    },
    MissingExternalBlindEvaluationReference,
    Serialization(String),
}

impl fmt::Display for SplitValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "benchmark split validation failed: {self:?}")
    }
}

impl std::error::Error for SplitValidationError {}

fn validate_site_list(
    partition: BenchmarkPartition,
    site_ids: &[String],
) -> Result<BTreeSet<String>, SplitValidationError> {
    let mut unique = BTreeSet::new();
    for site_id in site_ids {
        if site_id.trim().is_empty() {
            return Err(SplitValidationError::EmptySiteId);
        }
        if !unique.insert(site_id.clone()) {
            return Err(SplitValidationError::DuplicateSiteIdWithinPartition {
                partition,
                site_id: site_id.clone(),
            });
        }
    }
    if unique.is_empty() {
        return Err(SplitValidationError::EmptySitePartition(partition));
    }
    Ok(unique)
}

fn validate_sample_assignments(
    samples: &[BenchmarkSampleAssignment],
) -> Result<(usize, usize, usize), SplitValidationError> {
    if samples.is_empty() {
        return Err(SplitValidationError::MissingSampleAssignments);
    }

    let mut sample_ids = BTreeSet::new();
    let mut sites: BTreeMap<BenchmarkPartition, BTreeSet<String>> = BTreeMap::new();
    let mut counts: BTreeMap<BenchmarkPartition, usize> = BTreeMap::new();

    for sample in samples {
        if sample.sample_id.trim().is_empty() {
            return Err(SplitValidationError::EmptySampleId);
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(SplitValidationError::DuplicateSampleId(
                sample.sample_id.clone(),
            ));
        }
        if sample.site_id.trim().is_empty() {
            return Err(SplitValidationError::EmptySiteId);
        }
        sites
            .entry(sample.partition)
            .or_default()
            .insert(sample.site_id.clone());
        *counts.entry(sample.partition).or_default() += 1;
    }

    for partition in [
        BenchmarkPartition::Train,
        BenchmarkPartition::Validation,
        BenchmarkPartition::Test,
    ] {
        if !counts.contains_key(&partition) {
            return Err(SplitValidationError::EmptySitePartition(partition));
        }
    }

    Ok((
        sites
            .get(&BenchmarkPartition::Train)
            .map_or(0, BTreeSet::len),
        sites
            .get(&BenchmarkPartition::Validation)
            .map_or(0, BTreeSet::len),
        sites
            .get(&BenchmarkPartition::Test)
            .map_or(0, BTreeSet::len),
    ))
}

fn validate_site_partition_disjoint(
    samples: &[BenchmarkSampleAssignment],
) -> Result<(), SplitValidationError> {
    let mut partition_by_site: BTreeMap<&str, BenchmarkPartition> = BTreeMap::new();
    for sample in samples {
        if let Some(previous) = partition_by_site.insert(&sample.site_id, sample.partition) {
            if previous != sample.partition {
                return Err(SplitValidationError::SiteOverlapsPartitions {
                    site_id: sample.site_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn validate_single_holdout_group(
    samples: &[BenchmarkSampleAssignment],
    target: &str,
    field: fn(&BenchmarkSampleAssignment) -> Option<&str>,
    held_out_partition: BenchmarkPartition,
    mismatch: impl Fn(String, String) -> SplitValidationError,
    missing: impl Fn(String) -> SplitValidationError,
) -> Result<(), SplitValidationError> {
    for sample in samples {
        let value = field(sample)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| missing(sample.sample_id.clone()))?;
        let should_match = sample.partition == held_out_partition;
        if should_match != (value == target) {
            return Err(mismatch(sample.sample_id.clone(), target.to_owned()));
        }
    }
    Ok(())
}

/// Validate a benchmark split and return a deterministic, scope-limited receipt.
///
/// For held-out geography/data splits this checks that whole site IDs do not
/// cross train/validation/test. For within-site block splits it checks block
/// identity, partition coverage, an externally audited nearest cross-partition
/// distance for every block, and that those declared distances exceed the
/// preregistered spatial buffer. The external geometry audit is not recomputed.
pub fn validate_benchmark_split(
    manifest: &BenchmarkSplitManifest,
) -> Result<SplitValidationReceipt, SplitValidationError> {
    if manifest.schema_version != "luminous.heritage-benchmark-manifest.v0.1" {
        return Err(SplitValidationError::UnsupportedSchemaVersion);
    }
    if manifest.split_id.trim().is_empty() {
        return Err(SplitValidationError::EmptySplitId);
    }

    let mut train_site_count = 0;
    let mut validation_site_count = 0;
    let mut test_site_count = 0;
    let mut spatial_block_count = 0;
    let mut sample_count = 0;
    let mut minimum_declared_cross_partition_distance_m = None;
    let scope;

    match manifest.protocol {
        BenchmarkSplitProtocol::WithinSiteSpatialBlock => {
            let dependence = manifest
                .estimated_spatial_dependence_m
                .ok_or(SplitValidationError::MissingSpatialDependenceEstimate)?;
            if !dependence.is_finite() || dependence < 0.0 {
                return Err(SplitValidationError::InvalidSpatialDependenceEstimate);
            }
            let buffer = manifest
                .spatial_buffer_m
                .ok_or(SplitValidationError::MissingSpatialBuffer)?;
            if !buffer.is_finite() || buffer <= 0.0 {
                return Err(SplitValidationError::InvalidSpatialBuffer);
            }
            if buffer < dependence {
                return Err(SplitValidationError::SpatialBufferBelowDependence {
                    buffer_m: buffer,
                    estimated_dependence_m: dependence,
                });
            }
            if manifest
                .spatial_distance_audit_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(SplitValidationError::MissingSpatialDistanceAuditReference);
            }
            if manifest.spatial_block_assignments.is_empty() {
                return Err(SplitValidationError::EmptySpatialBlockAssignments);
            }

            let mut block_keys = BTreeSet::new();
            let mut site_sets: BTreeMap<BenchmarkPartition, BTreeSet<String>> = BTreeMap::new();
            let mut partition_counts: BTreeMap<BenchmarkPartition, usize> = BTreeMap::new();

            for block in &manifest.spatial_block_assignments {
                if block.site_id.trim().is_empty() {
                    return Err(SplitValidationError::EmptySpatialBlockSiteId);
                }
                if block.block_id.trim().is_empty() {
                    return Err(SplitValidationError::EmptySpatialBlockId);
                }
                if !block_keys.insert((block.site_id.clone(), block.block_id.clone())) {
                    return Err(SplitValidationError::DuplicateSpatialBlock {
                        site_id: block.site_id.clone(),
                        block_id: block.block_id.clone(),
                    });
                }

                let distance = block.nearest_other_partition_distance_m.ok_or_else(|| {
                    SplitValidationError::MissingCrossPartitionDistance {
                        site_id: block.site_id.clone(),
                        block_id: block.block_id.clone(),
                    }
                })?;
                if !distance.is_finite() || distance < 0.0 {
                    return Err(SplitValidationError::InvalidCrossPartitionDistance {
                        site_id: block.site_id.clone(),
                        block_id: block.block_id.clone(),
                    });
                }
                if distance < buffer {
                    return Err(SplitValidationError::CrossPartitionDistanceBelowBuffer {
                        site_id: block.site_id.clone(),
                        block_id: block.block_id.clone(),
                        distance_m: distance,
                        buffer_m: buffer,
                    });
                }

                minimum_declared_cross_partition_distance_m = Some(
                    minimum_declared_cross_partition_distance_m
                        .map_or(distance, |current: f64| current.min(distance)),
                );
                site_sets
                    .entry(block.partition)
                    .or_default()
                    .insert(block.site_id.clone());
                *partition_counts.entry(block.partition).or_default() += 1;
            }

            for partition in [
                BenchmarkPartition::Train,
                BenchmarkPartition::Validation,
                BenchmarkPartition::Test,
            ] {
                if !partition_counts.contains_key(&partition) {
                    return Err(SplitValidationError::MissingSpatialBlockPartition(
                        partition,
                    ));
                }
            }

            // Keep the manifest's site summary consistent with the detailed block assignment.
            for (partition, declared) in [
                (
                    BenchmarkPartition::Train,
                    validate_site_list(BenchmarkPartition::Train, &manifest.train_site_ids)?,
                ),
                (
                    BenchmarkPartition::Validation,
                    validate_site_list(
                        BenchmarkPartition::Validation,
                        &manifest.validation_site_ids,
                    )?,
                ),
                (
                    BenchmarkPartition::Test,
                    validate_site_list(BenchmarkPartition::Test, &manifest.test_site_ids)?,
                ),
            ] {
                if site_sets.get(&partition) != Some(&declared) {
                    return Err(SplitValidationError::SpatialBlockSiteListMismatch(
                        partition,
                    ));
                }
            }

            train_site_count = site_sets
                .get(&BenchmarkPartition::Train)
                .map_or(0, BTreeSet::len);
            validation_site_count = site_sets
                .get(&BenchmarkPartition::Validation)
                .map_or(0, BTreeSet::len);
            test_site_count = site_sets
                .get(&BenchmarkPartition::Test)
                .map_or(0, BTreeSet::len);
            spatial_block_count = manifest.spatial_block_assignments.len();
            scope = SplitValidationScope::DeclaredExternalSpatialBlockAudit;
        }
        BenchmarkSplitProtocol::HeldOutRegion
        | BenchmarkSplitProtocol::LeaveOneBiomeOut
        | BenchmarkSplitProtocol::LeaveOneSensorOut
        | BenchmarkSplitProtocol::TemporalHoldout => {
            let (train_sites, validation_sites, test_sites) =
                validate_sample_assignments(&manifest.sample_assignments)?;
            sample_count = manifest.sample_assignments.len();

            match manifest.protocol {
                BenchmarkSplitProtocol::HeldOutRegion => {
                    let target = manifest
                        .held_out_region_id
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or(SplitValidationError::MissingHeldOutRegionId)?;
                    validate_site_partition_disjoint(&manifest.sample_assignments)?;
                    validate_single_holdout_group(
                        &manifest.sample_assignments,
                        target,
                        |sample| sample.region_id.as_deref(),
                        BenchmarkPartition::Test,
                        |sample_id, expected| SplitValidationError::HeldOutRegionMismatch {
                            sample_id,
                            expected,
                        },
                        |sample_id| SplitValidationError::MissingRegionId { sample_id },
                    )?;
                }
                BenchmarkSplitProtocol::LeaveOneBiomeOut => {
                    let target = manifest
                        .held_out_biome_id
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or(SplitValidationError::MissingHeldOutBiomeId)?;
                    validate_site_partition_disjoint(&manifest.sample_assignments)?;
                    validate_single_holdout_group(
                        &manifest.sample_assignments,
                        target,
                        |sample| sample.biome_id.as_deref(),
                        BenchmarkPartition::Test,
                        |sample_id, expected| SplitValidationError::HeldOutBiomeMismatch {
                            sample_id,
                            expected,
                        },
                        |sample_id| SplitValidationError::MissingBiomeId { sample_id },
                    )?;
                }
                BenchmarkSplitProtocol::LeaveOneSensorOut => {
                    let target = manifest
                        .held_out_sensor_id
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                        .ok_or(SplitValidationError::MissingHeldOutSensorId)?;
                    validate_single_holdout_group(
                        &manifest.sample_assignments,
                        target,
                        |sample| sample.sensor_id.as_deref(),
                        BenchmarkPartition::Test,
                        |sample_id, expected| SplitValidationError::HeldOutSensorMismatch {
                            sample_id,
                            expected,
                        },
                        |sample_id| SplitValidationError::MissingSensorId { sample_id },
                    )?;
                }
                BenchmarkSplitProtocol::TemporalHoldout => {
                    let cutoff = manifest
                        .temporal_cutoff_at
                        .as_deref()
                        .and_then(parse_rfc3339_nanos)
                        .ok_or(SplitValidationError::MissingTemporalCutoff)?;
                    for sample in &manifest.sample_assignments {
                        let start = sample
                            .acquisition_start_at
                            .as_deref()
                            .and_then(parse_rfc3339_nanos);
                        let end = sample
                            .acquisition_end_at
                            .as_deref()
                            .and_then(parse_rfc3339_nanos);
                        let (Some(start), Some(end)) = (start, end) else {
                            return Err(SplitValidationError::MissingAcquisitionInterval {
                                sample_id: sample.sample_id.clone(),
                            });
                        };
                        if end < start {
                            return Err(SplitValidationError::InvalidSampleAcquisitionInterval {
                                sample_id: sample.sample_id.clone(),
                            });
                        }
                        let valid = match sample.partition {
                            BenchmarkPartition::Train | BenchmarkPartition::Validation => {
                                end <= cutoff
                            }
                            BenchmarkPartition::Test => start > cutoff,
                        };
                        if !valid {
                            return Err(SplitValidationError::TemporalPartitionViolation {
                                sample_id: sample.sample_id.clone(),
                                partition: sample.partition,
                            });
                        }
                    }
                }
                _ => unreachable!("protocol branch matched above"),
            }
            train_site_count = train_sites;
            validation_site_count = validation_sites;
            test_site_count = test_sites;
            scope = SplitValidationScope::SiteIdDisjointness;
        }
        BenchmarkSplitProtocol::ExternalBlind => {
            if manifest
                .external_blind_evaluation_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(SplitValidationError::MissingExternalBlindEvaluationReference);
            }
            let train = validate_site_list(BenchmarkPartition::Train, &manifest.train_site_ids)?;
            let validation = validate_site_list(
                BenchmarkPartition::Validation,
                &manifest.validation_site_ids,
            )?;
            let test = validate_site_list(BenchmarkPartition::Test, &manifest.test_site_ids)?;
            let mut seen = BTreeSet::new();
            for site_id in train.iter().chain(validation.iter()).chain(test.iter()) {
                if !seen.insert(site_id.clone()) {
                    return Err(SplitValidationError::SiteOverlapsPartitions {
                        site_id: site_id.clone(),
                    });
                }
            }
            train_site_count = train.len();
            validation_site_count = validation.len();
            test_site_count = test.len();
            scope = SplitValidationScope::SiteIdDisjointness;
        }
        BenchmarkSplitProtocol::HeldOutSite => {
            let train = validate_site_list(BenchmarkPartition::Train, &manifest.train_site_ids)?;
            let validation = validate_site_list(
                BenchmarkPartition::Validation,
                &manifest.validation_site_ids,
            )?;
            let test = validate_site_list(BenchmarkPartition::Test, &manifest.test_site_ids)?;

            let mut seen = BTreeSet::new();
            for site_id in train.iter().chain(validation.iter()).chain(test.iter()) {
                if !seen.insert(site_id.clone()) {
                    return Err(SplitValidationError::SiteOverlapsPartitions {
                        site_id: site_id.clone(),
                    });
                }
            }
            train_site_count = train.len();
            validation_site_count = validation.len();
            test_site_count = test.len();
            scope = SplitValidationScope::SiteIdDisjointness;
        }
    }

    let manifest_bytes = serde_json::to_vec(manifest)
        .map_err(|error| SplitValidationError::Serialization(error.to_string()))?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);

    #[derive(Serialize)]
    struct ReceiptDigest<'a> {
        schema_version: &'a str,
        split_id: &'a str,
        protocol: BenchmarkSplitProtocol,
        scope: SplitValidationScope,
        train_site_count: usize,
        validation_site_count: usize,
        test_site_count: usize,
        spatial_block_count: usize,
        sample_count: usize,
        manifest_sha256: &'a str,
        geometry_distances_recomputed_by_core: bool,
    }

    let receipt_payload = ReceiptDigest {
        schema_version: "luminous.heritage-split-validation-receipt.v0.1",
        split_id: &manifest.split_id,
        protocol: manifest.protocol,
        scope,
        train_site_count,
        validation_site_count,
        test_site_count,
        spatial_block_count,
        sample_count,
        manifest_sha256: &manifest_sha256,
        geometry_distances_recomputed_by_core: false,
    };
    let receipt_bytes = serde_json::to_vec(&receipt_payload)
        .map_err(|error| SplitValidationError::Serialization(error.to_string()))?;
    let receipt_sha256 = sha256_hex(&receipt_bytes);

    Ok(SplitValidationReceipt {
        schema_version: "luminous.heritage-split-validation-receipt.v0.1".to_owned(),
        split_id: manifest.split_id.clone(),
        protocol: manifest.protocol,
        scope,
        train_site_count,
        validation_site_count,
        test_site_count,
        spatial_block_count,
        sample_count,
        estimated_spatial_dependence_m: manifest.estimated_spatial_dependence_m,
        declared_spatial_buffer_m: manifest.spatial_buffer_m,
        minimum_declared_cross_partition_distance_m,
        spatial_distance_audit_ref: manifest.spatial_distance_audit_ref.clone(),
        geometry_distances_recomputed_by_core: false,
        manifest_sha256,
        receipt_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held_out_manifest() -> BenchmarkSplitManifest {
        BenchmarkSplitManifest {
            schema_version: "luminous.heritage-benchmark-manifest.v0.1".to_owned(),
            split_id: "held-out-sites-v1".to_owned(),
            protocol: BenchmarkSplitProtocol::HeldOutSite,
            train_site_ids: vec!["site-a".to_owned(), "site-b".to_owned()],
            validation_site_ids: vec!["site-c".to_owned()],
            test_site_ids: vec!["site-d".to_owned()],
            spatial_block_assignments: Vec::new(),
            sample_assignments: Vec::new(),
            held_out_region_id: None,
            held_out_biome_id: None,
            held_out_sensor_id: None,
            temporal_cutoff_at: None,
            external_blind_evaluation_ref: None,
            estimated_spatial_dependence_m: None,
            spatial_buffer_m: None,
            spatial_distance_audit_ref: None,
        }
    }

    fn sample(
        sample_id: &str,
        site_id: &str,
        partition: BenchmarkPartition,
        region_id: Option<&str>,
        biome_id: Option<&str>,
        sensor_id: Option<&str>,
        start_at: Option<&str>,
        end_at: Option<&str>,
    ) -> BenchmarkSampleAssignment {
        BenchmarkSampleAssignment {
            sample_id: sample_id.to_owned(),
            site_id: site_id.to_owned(),
            partition,
            region_id: region_id.map(str::to_owned),
            biome_id: biome_id.map(str::to_owned),
            sensor_id: sensor_id.map(str::to_owned),
            acquisition_start_at: start_at.map(str::to_owned),
            acquisition_end_at: end_at.map(str::to_owned),
        }
    }

    #[test]
    fn held_out_site_split_is_disjoint_and_receipt_is_deterministic() {
        let manifest = held_out_manifest();
        let first = validate_benchmark_split(&manifest).unwrap();
        let second = validate_benchmark_split(&manifest).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.scope, SplitValidationScope::SiteIdDisjointness);
        assert_eq!(first.train_site_count, 2);
        assert_eq!(first.validation_site_count, 1);
        assert_eq!(first.test_site_count, 1);
        assert_eq!(first.receipt_sha256.len(), 64);
        assert!(!first.geometry_distances_recomputed_by_core);
    }

    #[test]
    fn region_biome_and_sensor_holdouts_validate_explicit_sample_groups() {
        let assignments = || {
            vec![
                sample(
                    "a-train",
                    "site-a",
                    BenchmarkPartition::Train,
                    Some("region-a"),
                    Some("biome-a"),
                    Some("sensor-a"),
                    None,
                    None,
                ),
                sample(
                    "b-valid",
                    "site-b",
                    BenchmarkPartition::Validation,
                    Some("region-b"),
                    Some("biome-b"),
                    Some("sensor-b"),
                    None,
                    None,
                ),
                sample(
                    "c-test",
                    "site-c",
                    BenchmarkPartition::Test,
                    Some("region-c"),
                    Some("biome-c"),
                    Some("sensor-c"),
                    None,
                    None,
                ),
            ]
        };

        let mut region = held_out_manifest();
        region.protocol = BenchmarkSplitProtocol::HeldOutRegion;
        region.held_out_region_id = Some("region-c".to_owned());
        region.sample_assignments = assignments();
        assert_eq!(validate_benchmark_split(&region).unwrap().sample_count, 3);

        let mut biome = held_out_manifest();
        biome.protocol = BenchmarkSplitProtocol::LeaveOneBiomeOut;
        biome.held_out_biome_id = Some("biome-c".to_owned());
        biome.sample_assignments = assignments();
        assert_eq!(validate_benchmark_split(&biome).unwrap().sample_count, 3);

        let mut sensor = held_out_manifest();
        sensor.protocol = BenchmarkSplitProtocol::LeaveOneSensorOut;
        sensor.held_out_sensor_id = Some("sensor-c".to_owned());
        sensor.sample_assignments = assignments();
        assert_eq!(validate_benchmark_split(&sensor).unwrap().sample_count, 3);
    }

    #[test]
    fn held_out_region_rejects_a_site_leaking_across_partitions() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::HeldOutRegion;
        manifest.held_out_region_id = Some("region-c".to_owned());
        manifest.sample_assignments = vec![
            sample(
                "train-a",
                "site-shared",
                BenchmarkPartition::Train,
                Some("region-a"),
                Some("biome-a"),
                Some("sensor-a"),
                None,
                None,
            ),
            sample(
                "test-c",
                "site-shared",
                BenchmarkPartition::Test,
                Some("region-c"),
                Some("biome-c"),
                Some("sensor-c"),
                None,
                None,
            ),
            sample(
                "validation-b",
                "site-b",
                BenchmarkPartition::Validation,
                Some("region-b"),
                Some("biome-b"),
                Some("sensor-b"),
                None,
                None,
            ),
        ];
        assert_eq!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::SiteOverlapsPartitions {
                site_id: "site-shared".to_owned()
            })
        );
    }

    #[test]
    fn temporal_holdout_allows_same_site_but_not_time_leakage() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::TemporalHoldout;
        manifest.temporal_cutoff_at = Some("2021-01-01T00:00:00Z".to_owned());
        manifest.sample_assignments = vec![
            sample(
                "train-2020",
                "site-a",
                BenchmarkPartition::Train,
                Some("region-a"),
                Some("biome-a"),
                Some("sensor-a"),
                Some("2020-01-01T00:00:00Z"),
                Some("2020-12-31T23:59:59Z"),
            ),
            sample(
                "validation-2020",
                "site-a",
                BenchmarkPartition::Validation,
                Some("region-a"),
                Some("biome-a"),
                Some("sensor-a"),
                Some("2020-03-01T00:00:00Z"),
                Some("2020-11-01T00:00:00Z"),
            ),
            sample(
                "test-2021",
                "site-a",
                BenchmarkPartition::Test,
                Some("region-a"),
                Some("biome-a"),
                Some("sensor-a"),
                Some("2021-02-01T00:00:00Z"),
                Some("2021-03-01T00:00:00Z"),
            ),
        ];
        assert_eq!(validate_benchmark_split(&manifest).unwrap().sample_count, 3);

        manifest.sample_assignments[2].acquisition_start_at =
            Some("2020-12-31T00:00:00Z".to_owned());
        assert!(matches!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::TemporalPartitionViolation {
                sample_id,
                partition: BenchmarkPartition::Test
            }) if sample_id == "test-2021"
        ));
    }

    #[test]
    fn external_blind_requires_a_separate_evaluation_reference() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::ExternalBlind;
        assert_eq!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::MissingExternalBlindEvaluationReference)
        );
        manifest.external_blind_evaluation_ref =
            Some("urn:synthetic:blind-evaluation:1".to_owned());
        let receipt = validate_benchmark_split(&manifest).unwrap();
        assert_eq!(receipt.protocol, BenchmarkSplitProtocol::ExternalBlind);
    }

    #[test]
    fn held_out_site_overlap_fails_closed() {
        let mut manifest = held_out_manifest();
        manifest.test_site_ids.push("site-b".to_owned());
        assert_eq!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::SiteOverlapsPartitions {
                site_id: "site-b".to_owned(),
            })
        );
    }

    #[test]
    fn duplicate_site_within_one_partition_is_rejected() {
        let mut manifest = held_out_manifest();
        manifest.train_site_ids.push("site-a".to_owned());
        assert_eq!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::DuplicateSiteIdWithinPartition {
                partition: BenchmarkPartition::Train,
                site_id: "site-a".to_owned(),
            })
        );
    }

    #[test]
    fn within_site_blocks_require_dependence_buffer_and_external_audit() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::WithinSiteSpatialBlock;
        manifest.train_site_ids = vec!["site-x".to_owned()];
        manifest.validation_site_ids = vec!["site-x".to_owned()];
        manifest.test_site_ids = vec!["site-x".to_owned()];
        manifest.estimated_spatial_dependence_m = Some(20.0);
        manifest.spatial_buffer_m = Some(25.0);
        manifest.spatial_distance_audit_ref = Some("urn:synthetic:spatial-audit:1".to_owned());
        manifest.spatial_block_assignments = vec![
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "block-a".to_owned(),
                partition: BenchmarkPartition::Train,
                nearest_other_partition_distance_m: Some(30.0),
            },
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "block-b".to_owned(),
                partition: BenchmarkPartition::Validation,
                nearest_other_partition_distance_m: Some(30.0),
            },
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "block-c".to_owned(),
                partition: BenchmarkPartition::Test,
                nearest_other_partition_distance_m: Some(28.0),
            },
        ];
        let receipt = validate_benchmark_split(&manifest).unwrap();
        assert_eq!(
            receipt.scope,
            SplitValidationScope::DeclaredExternalSpatialBlockAudit
        );
        assert_eq!(receipt.train_site_count, 1);
        assert_eq!(receipt.validation_site_count, 1);
        assert_eq!(receipt.test_site_count, 1);
        assert_eq!(receipt.spatial_block_count, 3);
        assert_eq!(
            receipt.minimum_declared_cross_partition_distance_m,
            Some(28.0)
        );
        assert!(!receipt.geometry_distances_recomputed_by_core);
    }

    #[test]
    fn spatial_buffer_below_dependence_estimate_is_rejected() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::WithinSiteSpatialBlock;
        manifest.estimated_spatial_dependence_m = Some(100.0);
        manifest.spatial_buffer_m = Some(80.0);
        manifest.spatial_distance_audit_ref = Some("urn:synthetic:audit".to_owned());
        assert_eq!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::SpatialBufferBelowDependence {
                buffer_m: 80.0,
                estimated_dependence_m: 100.0,
            })
        );
    }

    #[test]
    fn spatial_block_too_close_to_other_partition_is_rejected() {
        let mut manifest = held_out_manifest();
        manifest.protocol = BenchmarkSplitProtocol::WithinSiteSpatialBlock;
        manifest.train_site_ids = vec!["site-x".to_owned()];
        manifest.validation_site_ids = vec!["site-x".to_owned()];
        manifest.test_site_ids = vec!["site-x".to_owned()];
        manifest.estimated_spatial_dependence_m = Some(10.0);
        manifest.spatial_buffer_m = Some(20.0);
        manifest.spatial_distance_audit_ref = Some("urn:synthetic:audit".to_owned());
        manifest.spatial_block_assignments = vec![
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "train".to_owned(),
                partition: BenchmarkPartition::Train,
                nearest_other_partition_distance_m: Some(15.0),
            },
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "valid".to_owned(),
                partition: BenchmarkPartition::Validation,
                nearest_other_partition_distance_m: Some(15.0),
            },
            SpatialBlockAssignment {
                site_id: "site-x".to_owned(),
                block_id: "test".to_owned(),
                partition: BenchmarkPartition::Test,
                nearest_other_partition_distance_m: Some(25.0),
            },
        ];
        assert!(matches!(
            validate_benchmark_split(&manifest),
            Err(SplitValidationError::CrossPartitionDistanceBelowBuffer { .. })
        ));
    }
}
