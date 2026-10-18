// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic surface-water index core for already aligned raster samples.
//!
//! This does not download, reproject, resample, or mask real rasters. Callers
//! must supply surface reflectance on a declared common grid and apply the
//! product-specific cloud/shadow/invalid-pixel QA masks before estimation.

use crate::utility_telemetry::{
    Coordinates, DataTransport, MetricKind, ObservationProvenance, ObservationQuality,
    ObservationSource, ObservationValidationError, ObservationValue, QualityFlag, SubjectKind,
    UtilityDomain, UtilityObservation,
};
use std::fmt;

const DENOMINATOR_EPSILON: f64 = 1.0e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterIndex {
    Ndwi,
    Mndwi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelQuality {
    Clear,
    Cloud,
    CloudShadow,
    SnowOrIce,
    NoData,
    OtherInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReflectancePixel {
    pub green: f64,
    pub near_infrared: f64,
    pub shortwave_infrared_1: Option<f64>,
    /// Ground area represented by this pixel after reprojection/resampling.
    pub area_m2: f64,
    /// Must come from the chosen provider/product's QA-mask policy.
    pub quality: PixelQuality,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceWaterConfig {
    pub index: WaterIndex,
    /// Must be calibrated and recorded with the scene/product and study area.
    pub threshold: f64,
    /// Do not publish an area estimate when clear-index coverage is below this.
    pub minimum_valid_area_fraction: f64,
}

impl SurfaceWaterConfig {
    pub fn validate(self) -> Result<(), SurfaceWaterError> {
        if !self.threshold.is_finite() || !(-1.0..=1.0).contains(&self.threshold) {
            return Err(SurfaceWaterError::InvalidThreshold);
        }
        if !self.minimum_valid_area_fraction.is_finite()
            || self.minimum_valid_area_fraction <= 0.0
            || self.minimum_valid_area_fraction > 1.0
        {
            return Err(SurfaceWaterError::InvalidCoverageThreshold);
        }
        Ok(())
    }
}

/// Metadata required to turn an area estimate into a provenance-bearing
/// civic-utility observation. The caller must supply the effective observation
/// time and upstream scene/product identifiers; they are never guessed here.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceWaterObservationContext {
    pub observation_id: String,
    pub subject_id: String,
    pub subject_kind: SubjectKind,
    pub observed_at_unix_ms: u64,
    pub received_at_unix_ms: u64,
    pub source_id: String,
    pub processing_version: String,
    /// SHA-256 of the immutable source/processing evidence bundle; canonical lowercase hex.
    pub evidence_sha256: String,
    pub upstream_observation_ids: Vec<String>,
    pub location: Option<Coordinates>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceWaterEstimate {
    pub index: WaterIndex,
    pub threshold: f64,
    pub minimum_valid_area_fraction: f64,
    pub total_pixel_count: usize,
    pub water_pixel_count: usize,
    pub valid_pixel_count: usize,
    pub masked_pixel_count: usize,
    pub invalid_index_pixel_count: usize,
    pub total_area_m2: f64,
    pub valid_area_m2: f64,
    pub masked_or_invalid_area_m2: f64,
    /// Classified water area within valid pixels only. Masked or invalid regions
    /// are never extrapolated; None when valid coverage is below the configured minimum.
    pub classified_water_area_m2: Option<f64>,
    pub valid_area_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceWaterError {
    InvalidThreshold,
    InvalidCoverageThreshold,
    InvalidPixelArea { index: usize },
    AreaOverflow,
    InsufficientValidCoverage,
    MissingUpstreamLineage,
    InvalidUpstreamLineage,
    MissingProcessingVersion,
    MissingEvidenceDigest,
    InvalidEvidenceDigest,
    InvalidObservation(Vec<ObservationValidationError>),
}

impl fmt::Display for SurfaceWaterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SurfaceWaterError {}

pub fn estimate_surface_water(
    pixels: &[ReflectancePixel],
    config: SurfaceWaterConfig,
) -> Result<SurfaceWaterEstimate, SurfaceWaterError> {
    config.validate()?;

    let mut total_area_m2 = 0.0;
    let mut valid_area_m2 = 0.0;
    let mut masked_or_invalid_area_m2 = 0.0;
    let mut water_area_m2_accumulator = 0.0;
    let mut water_pixel_count = 0;
    let mut valid_pixel_count = 0;
    let mut masked_pixel_count = 0;
    let mut invalid_index_pixel_count = 0;

    for (index, pixel) in pixels.iter().enumerate() {
        if !pixel.area_m2.is_finite() || pixel.area_m2 <= 0.0 {
            return Err(SurfaceWaterError::InvalidPixelArea { index });
        }
        total_area_m2 += pixel.area_m2;
        if !total_area_m2.is_finite() {
            return Err(SurfaceWaterError::AreaOverflow);
        }

        if pixel.quality != PixelQuality::Clear {
            masked_pixel_count += 1;
            masked_or_invalid_area_m2 += pixel.area_m2;
            continue;
        }

        let Some(index_value) = calculate_index(*pixel, config.index) else {
            invalid_index_pixel_count += 1;
            masked_or_invalid_area_m2 += pixel.area_m2;
            continue;
        };

        valid_pixel_count += 1;
        valid_area_m2 += pixel.area_m2;
        if index_value >= config.threshold {
            water_pixel_count += 1;
            water_area_m2_accumulator += pixel.area_m2;
        }
    }

    let valid_area_fraction = if total_area_m2 > 0.0 {
        valid_area_m2 / total_area_m2
    } else {
        0.0
    };
    let classified_water_area_m2 = if valid_area_fraction >= config.minimum_valid_area_fraction {
        Some(water_area_m2_accumulator)
    } else {
        None
    };

    Ok(SurfaceWaterEstimate {
        index: config.index,
        threshold: config.threshold,
        minimum_valid_area_fraction: config.minimum_valid_area_fraction,
        total_pixel_count: pixels.len(),
        water_pixel_count,
        valid_pixel_count,
        masked_pixel_count,
        invalid_index_pixel_count,
        total_area_m2,
        valid_area_m2,
        masked_or_invalid_area_m2,
        classified_water_area_m2,
        valid_area_fraction,
    })
}

impl SurfaceWaterEstimate {
    /// Convert a qualified area estimate into the existing civic telemetry
    /// contract. This emits a derived observation, never an in-situ measurement.
    /// The output is validated before it is returned.
    pub fn to_utility_observation(
        &self,
        context: SurfaceWaterObservationContext,
    ) -> Result<UtilityObservation, SurfaceWaterError> {
        let classified_water_area_m2 = self
            .classified_water_area_m2
            .ok_or(SurfaceWaterError::InsufficientValidCoverage)?;
        if context.upstream_observation_ids.is_empty() {
            return Err(SurfaceWaterError::MissingUpstreamLineage);
        }
        if context
            .upstream_observation_ids
            .iter()
            .any(|id| id.trim().is_empty())
        {
            return Err(SurfaceWaterError::InvalidUpstreamLineage);
        }
        if context.processing_version.trim().is_empty() {
            return Err(SurfaceWaterError::MissingProcessingVersion);
        }
        if context.evidence_sha256.trim().is_empty() {
            return Err(SurfaceWaterError::MissingEvidenceDigest);
        }
        if !is_canonical_sha256(&context.evidence_sha256) {
            return Err(SurfaceWaterError::InvalidEvidenceDigest);
        }

        let partial_spatial_coverage = self.valid_pixel_count < self.total_pixel_count;
        let mut quality_flags = vec![QualityFlag::ModelDerived];
        if partial_spatial_coverage {
            quality_flags.push(QualityFlag::PartialSpatialCoverage);
        }

        let observation = UtilityObservation {
            schema_version: 1,
            observation_id: context.observation_id,
            subject_id: context.subject_id,
            subject_kind: context.subject_kind,
            domain: UtilityDomain::Water,
            metric: MetricKind::SurfaceWaterExtent,
            value: ObservationValue::Numeric {
                value: classified_water_area_m2,
                unit: crate::utility_telemetry::MeasurementUnit::SquareMetre,
            },
            observed_at_unix_ms: context.observed_at_unix_ms,
            received_at_unix_ms: context.received_at_unix_ms,
            source: ObservationSource::DerivedEstimate,
            transport: DataTransport::BatchImport,
            provenance: ObservationProvenance {
                source_id: context.source_id,
                source_version: Some(context.processing_version),
                evidence_sha256: Some(context.evidence_sha256),
                upstream_observation_ids: context.upstream_observation_ids,
                processing_parameters: Some(serde_json::json!({
                    "index": match self.index {
                        WaterIndex::Ndwi => "ndwi",
                        WaterIndex::Mndwi => "mndwi",
                    },
                    "area_reporting_policy": "valid_pixels_only_no_extrapolation",
                    "partial_spatial_coverage": partial_spatial_coverage,
                    "complete_study_area_coverage": !partial_spatial_coverage,
                    "threshold": self.threshold,
                    "minimum_valid_area_fraction": self.minimum_valid_area_fraction,
                    "valid_area_fraction": self.valid_area_fraction,
                    "total_pixel_count": self.total_pixel_count,
                    "valid_pixel_count": self.valid_pixel_count,
                    "water_pixel_count": self.water_pixel_count,
                    "masked_pixel_count": self.masked_pixel_count,
                    "invalid_index_pixel_count": self.invalid_index_pixel_count,
                })),
            },
            quality: ObservationQuality {
                confidence: None,
                uncertainty: None,
                flags: quality_flags,
            },
            location: context.location,
        };
        observation
            .validate()
            .map_err(SurfaceWaterError::InvalidObservation)?;
        Ok(observation)
    }
}

fn is_canonical_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn calculate_index(pixel: ReflectancePixel, index: WaterIndex) -> Option<f64> {
    if !pixel.green.is_finite() || !pixel.near_infrared.is_finite() {
        return None;
    }
    let numerator;
    let denominator;
    match index {
        WaterIndex::Ndwi => {
            numerator = pixel.green - pixel.near_infrared;
            denominator = pixel.green + pixel.near_infrared;
        }
        WaterIndex::Mndwi => {
            let swir = pixel.shortwave_infrared_1?;
            if !swir.is_finite() {
                return None;
            }
            numerator = pixel.green - swir;
            denominator = pixel.green + swir;
        }
    }
    if !denominator.is_finite() || denominator <= DENOMINATOR_EPSILON {
        return None;
    }
    let index_value = numerator / denominator;
    (index_value.is_finite() && (-1.0..=1.0).contains(&index_value)).then_some(index_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(
        green: f64,
        nir: f64,
        swir: Option<f64>,
        area: f64,
        quality: PixelQuality,
    ) -> ReflectancePixel {
        ReflectancePixel {
            green,
            near_infrared: nir,
            shortwave_infrared_1: swir,
            area_m2: area,
            quality,
        }
    }

    fn config(index: WaterIndex) -> SurfaceWaterConfig {
        SurfaceWaterConfig {
            index,
            threshold: 0.2,
            minimum_valid_area_fraction: 0.5,
        }
    }

    #[test]
    fn ndwi_estimates_weighted_area_only_from_valid_clear_pixels() {
        let pixels = [
            pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear),
            pixel(0.1, 0.3, Some(0.25), 100.0, PixelQuality::Clear),
            pixel(0.5, 0.1, Some(0.03), 200.0, PixelQuality::Cloud),
        ];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        assert_eq!(estimate.total_pixel_count, 3);
        assert_eq!(estimate.valid_pixel_count, 2);
        assert_eq!(estimate.masked_pixel_count, 1);
        assert_eq!(estimate.water_pixel_count, 1);
        assert_eq!(estimate.classified_water_area_m2, Some(100.0));
        assert!((estimate.valid_area_fraction - 0.5).abs() < 1e-12);
    }

    #[test]
    fn mndwi_requires_swir_and_low_coverage_withholds_estimate() {
        let pixels = [
            pixel(0.3, 0.1, None, 100.0, PixelQuality::Clear),
            pixel(0.1, 0.3, Some(0.4), 100.0, PixelQuality::Clear),
            pixel(0.5, 0.1, Some(0.03), 200.0, PixelQuality::Cloud),
        ];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Mndwi)).unwrap();
        assert_eq!(estimate.invalid_index_pixel_count, 1);
        assert_eq!(estimate.classified_water_area_m2, None);
    }

    #[test]
    fn zero_denominator_is_invalid_not_dry_water() {
        let pixels = [pixel(0.0, 0.0, Some(0.0), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        assert_eq!(estimate.invalid_index_pixel_count, 1);
        assert_eq!(estimate.classified_water_area_m2, None);
        assert_eq!(estimate.valid_area_fraction, 0.0);
    }

    #[test]
    fn out_of_range_normalized_index_is_invalid_not_water() {
        let pixels = [pixel(0.8, -0.2, Some(0.1), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        assert_eq!(estimate.invalid_index_pixel_count, 1);
        assert_eq!(estimate.classified_water_area_m2, None);
    }

    #[test]
    fn invalid_pixel_area_fails_closed() {
        let pixels = [pixel(0.3, 0.1, Some(0.05), 0.0, PixelQuality::Clear)];
        assert_eq!(
            estimate_surface_water(&pixels, config(WaterIndex::Ndwi)),
            Err(SurfaceWaterError::InvalidPixelArea { index: 0 })
        );
    }

    fn observation_context() -> SurfaceWaterObservationContext {
        SurfaceWaterObservationContext {
            observation_id: "derived-water-area-1".into(),
            subject_id: "vaal-dam-study-area".into(),
            subject_kind: SubjectKind::Reservoir,
            observed_at_unix_ms: 1_000,
            received_at_unix_ms: 1_100,
            source_id: "sol-atlas-water-index".into(),
            processing_version: "surface-water-index-v1".into(),
            evidence_sha256: "a".repeat(64),
            upstream_observation_ids: vec!["sentinel-scene-001".into()],
            location: Some(Coordinates {
                latitude_deg: -26.9,
                longitude_deg: 28.1,
            }),
        }
    }

    #[test]
    fn qualified_area_becomes_validated_derived_telemetry() {
        let pixels = [pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        let observation = estimate
            .to_utility_observation(observation_context())
            .unwrap();
        assert_eq!(observation.validate(), Ok(()));
        assert_eq!(observation.metric, MetricKind::SurfaceWaterExtent);
        assert_eq!(observation.source, ObservationSource::DerivedEstimate);
        assert!(
            !observation
                .quality
                .flags
                .contains(&QualityFlag::PartialSpatialCoverage)
        );
        assert_eq!(
            observation.provenance.upstream_observation_ids,
            vec!["sentinel-scene-001"]
        );
        assert_eq!(
            observation.provenance.source_version.as_deref(),
            Some("surface-water-index-v1")
        );
        let parameters = observation
            .provenance
            .processing_parameters
            .as_ref()
            .expect("index parameters must be captured");
        assert_eq!(parameters["index"], "ndwi");
        assert_eq!(parameters["threshold"], 0.2);
        assert_eq!(parameters["minimum_valid_area_fraction"], 0.5);
        assert_eq!(parameters["valid_area_fraction"], 1.0);
        assert_eq!(parameters["water_pixel_count"], 1);
        assert_eq!(observation.observed_at_unix_ms, 1_000);
    }

    #[test]
    fn partial_coverage_is_flagged_and_never_extrapolated_to_the_full_area() {
        let pixels = [
            pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear),
            pixel(0.5, 0.1, Some(0.03), 300.0, PixelQuality::Cloud),
        ];
        let mut cfg = config(WaterIndex::Ndwi);
        cfg.minimum_valid_area_fraction = 0.25;

        let estimate = estimate_surface_water(&pixels, cfg).unwrap();
        assert_eq!(estimate.total_area_m2, 400.0);
        assert_eq!(estimate.valid_area_m2, 100.0);
        assert_eq!(estimate.valid_area_fraction, 0.25);
        // The masked 300 m² is unknown, not assumed dry and not imputed as water.
        assert_eq!(estimate.classified_water_area_m2, Some(100.0));

        let observation = estimate
            .to_utility_observation(observation_context())
            .unwrap();
        assert!(
            observation
                .quality
                .flags
                .contains(&QualityFlag::PartialSpatialCoverage)
        );
        let parameters = observation
            .provenance
            .processing_parameters
            .as_ref()
            .expect("processing semantics must be retained");
        assert_eq!(
            parameters["area_reporting_policy"],
            "valid_pixels_only_no_extrapolation"
        );
        assert_eq!(parameters["partial_spatial_coverage"], true);
        assert_eq!(parameters["complete_study_area_coverage"], false);
        assert_eq!(
            observation.value,
            ObservationValue::Numeric {
                value: 100.0,
                unit: crate::utility_telemetry::MeasurementUnit::SquareMetre
            }
        );
        assert!(observation.validate().is_ok());
    }

    #[test]
    fn low_coverage_and_missing_lineage_cannot_become_observations() {
        let pixels = [pixel(0.3, 0.1, None, 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Mndwi)).unwrap();
        assert_eq!(
            estimate.to_utility_observation(observation_context()),
            Err(SurfaceWaterError::InsufficientValidCoverage)
        );

        let pixels = [pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        let mut context = observation_context();
        context.upstream_observation_ids.clear();
        assert_eq!(
            estimate.to_utility_observation(context),
            Err(SurfaceWaterError::MissingUpstreamLineage)
        );
    }

    #[test]
    fn derived_water_observation_requires_canonical_evidence_digest() {
        let pixels = [pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();

        let mut context = observation_context();
        context.evidence_sha256.clear();
        assert_eq!(
            estimate.to_utility_observation(context),
            Err(SurfaceWaterError::MissingEvidenceDigest)
        );

        let mut context = observation_context();
        context.evidence_sha256 = "not-a-digest".into();
        assert_eq!(
            estimate.to_utility_observation(context),
            Err(SurfaceWaterError::InvalidEvidenceDigest)
        );

        let mut context = observation_context();
        context.evidence_sha256 = "A".repeat(64);
        assert_eq!(
            estimate.to_utility_observation(context),
            Err(SurfaceWaterError::InvalidEvidenceDigest)
        );
    }

    #[test]
    fn rejects_blank_upstream_observation_identifiers() {
        let pixels = [pixel(0.3, 0.1, Some(0.05), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        let mut context = observation_context();
        context.upstream_observation_ids = vec!["   ".into()];
        assert_eq!(
            estimate.to_utility_observation(context),
            Err(SurfaceWaterError::InvalidUpstreamLineage)
        );
    }

    #[test]
    fn thresholds_are_explicit_and_validated() {
        let mut cfg = config(WaterIndex::Ndwi);
        cfg.threshold = f64::NAN;
        assert_eq!(cfg.validate(), Err(SurfaceWaterError::InvalidThreshold));

        cfg = config(WaterIndex::Ndwi);
        cfg.minimum_valid_area_fraction = 0.0;
        assert_eq!(
            cfg.validate(),
            Err(SurfaceWaterError::InvalidCoverageThreshold)
        );
    }
}
