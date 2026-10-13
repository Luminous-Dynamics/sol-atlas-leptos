// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic surface-water index core for already aligned raster samples.
//!
//! This does not download, reproject, resample, or mask real rasters. Callers
//! must supply surface reflectance on a declared common grid and apply the
//! product-specific cloud/shadow/invalid-pixel QA masks before estimation.

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

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceWaterEstimate {
    pub index: WaterIndex,
    pub threshold: f64,
    pub total_pixel_count: usize,
    pub water_pixel_count: usize,
    pub valid_pixel_count: usize,
    pub masked_pixel_count: usize,
    pub invalid_index_pixel_count: usize,
    pub total_area_m2: f64,
    pub valid_area_m2: f64,
    pub masked_or_invalid_area_m2: f64,
    /// None when valid clear-pixel coverage is below the configured minimum.
    pub water_area_m2: Option<f64>,
    pub valid_area_fraction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceWaterError {
    InvalidThreshold,
    InvalidCoverageThreshold,
    InvalidPixelArea { index: usize },
    AreaOverflow,
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
    let water_area_m2 = if valid_area_fraction >= config.minimum_valid_area_fraction {
        Some(water_area_m2_accumulator)
    } else {
        None
    };

    Ok(SurfaceWaterEstimate {
        index: config.index,
        threshold: config.threshold,
        total_pixel_count: pixels.len(),
        water_pixel_count,
        valid_pixel_count,
        masked_pixel_count,
        invalid_index_pixel_count,
        total_area_m2,
        valid_area_m2,
        masked_or_invalid_area_m2,
        water_area_m2,
        valid_area_fraction,
    })
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
    index_value.is_finite().then_some(index_value)
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
        assert_eq!(estimate.water_area_m2, Some(100.0));
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
        assert_eq!(estimate.water_area_m2, None);
    }

    #[test]
    fn zero_denominator_is_invalid_not_dry_water() {
        let pixels = [pixel(0.0, 0.0, Some(0.0), 100.0, PixelQuality::Clear)];
        let estimate = estimate_surface_water(&pixels, config(WaterIndex::Ndwi)).unwrap();
        assert_eq!(estimate.invalid_index_pixel_count, 1);
        assert_eq!(estimate.water_area_m2, None);
        assert_eq!(estimate.valid_area_fraction, 0.0);
    }

    #[test]
    fn invalid_pixel_area_fails_closed() {
        let pixels = [pixel(0.3, 0.1, Some(0.05), 0.0, PixelQuality::Clear)];
        assert_eq!(
            estimate_surface_water(&pixels, config(WaterIndex::Ndwi)),
            Err(SurfaceWaterError::InvalidPixelArea { index: 0 })
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
