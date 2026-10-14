// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Product-specific quality decoding for optical Earth-observation pixels.
//!
//! This module decodes QA values only. It does not open raster files, align
//! grids, perform reprojection, or infer that an asset uses a particular
//! product version. The caller must select the policy from scene metadata and
//! record the version with the resulting observation.

use crate::surface_water::PixelQuality;

/// Version tag for the conservative Sentinel-2 Level-2A SCL/data-mask policy.
pub const SENTINEL2_L2A_SCL_POLICY_VERSION: &str = "sentinel2-l2a-scl-v1";

/// Version tag for the conservative Landsat 8 Collection 2 QA_PIXEL/QA_RADSAT policy.
pub const LANDSAT8_C2_QA_POLICY_VERSION: &str = "landsat8-c2-qa-v1";

/// USGS Landsat Collection 2 Level-2 surface reflectance scale and offset.
pub const LANDSAT_C2_SR_SCALE_FACTOR: f64 = 0.000_027_5;
pub const LANDSAT_C2_SR_ADD_OFFSET: f64 = -0.2;

/// Water-index bands whose saturation state must invalidate a Landsat pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landsat8WaterBandSet {
    /// Green (Band 3) and near-infrared (Band 5).
    NdwiGreenNir,
    /// Green (Band 3) and shortwave-infrared 1 (Band 6).
    MndwiGreenSwir1,
}

impl Landsat8WaterBandSet {
    /// Bit mask for the required optical bands in Landsat 8/9 QA_RADSAT.
    fn required_saturation_mask(self) -> u16 {
        const BAND_3_BIT: u16 = 1 << 2;
        const BAND_5_BIT: u16 = 1 << 4;
        const BAND_6_BIT: u16 = 1 << 5;

        match self {
            Self::NdwiGreenNir => BAND_3_BIT | BAND_5_BIT,
            Self::MndwiGreenSwir1 => BAND_3_BIT | BAND_6_BIT,
        }
    }
}

/// Confidence threshold for the two-bit cloud, shadow, snow and cirrus fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landsat8ConfidencePolicy {
    /// Mask a condition when its encoded confidence is medium/reserved (2) or high (3).
    RejectMediumOrHigh,
    /// Mask a condition only at high confidence (3); suitable only when a
    /// documented comparison justifies the less conservative mask.
    RejectHighOnly,
}

impl Landsat8ConfidencePolicy {
    fn rejects(self, confidence: u16) -> bool {
        match self {
            Self::RejectMediumOrHigh => confidence >= 2,
            Self::RejectHighOnly => confidence == 3,
        }
    }
}

/// Conservative default: mask medium/high confidence conditions.
impl Default for Landsat8ConfidencePolicy {
    fn default() -> Self {
        Self::RejectMediumOrHigh
    }
}

/// Apply the USGS Landsat 8/9 Collection 2 QA_PIXEL and QA_RADSAT semantics
/// to one pixel for the selected index's required bands.
///
/// Water flag (QA_PIXEL bit 7) is intentionally not rejected because water
/// pixels are the target of this analysis. The QA_PIXEL "clear" bit is not
/// trusted as the only quality gate: individual cloud/shadow/snow/cirrus
/// flags and the selected bands' saturation bits are checked explicitly.
pub fn classify_landsat8_c2_pixel(
    qa_pixel: u16,
    qa_radsat: u16,
    bands: Landsat8WaterBandSet,
    confidence_policy: Landsat8ConfidencePolicy,
) -> PixelQuality {
    const FILL: u16 = 1 << 0;
    const DILATED_CLOUD: u16 = 1 << 1;
    const CIRRUS: u16 = 1 << 2;
    const CLOUD: u16 = 1 << 3;
    const CLOUD_SHADOW: u16 = 1 << 4;
    const SNOW: u16 = 1 << 5;
    const WATER: u16 = 1 << 7;
    const TERRAIN_OCCLUSION: u16 = 1 << 11;

    if qa_pixel & FILL != 0 {
        return PixelQuality::NoData;
    }

    // A water flag is informative, not a reason to mask a water-index sample.
    let _is_water = qa_pixel & WATER != 0;

    let cloud_confidence = (qa_pixel >> 8) & 0b11;
    let shadow_confidence = (qa_pixel >> 10) & 0b11;
    let snow_confidence = (qa_pixel >> 12) & 0b11;
    let cirrus_confidence = (qa_pixel >> 14) & 0b11;

    if qa_pixel & (DILATED_CLOUD | CIRRUS | CLOUD) != 0
        || confidence_policy.rejects(cloud_confidence)
        || confidence_policy.rejects(cirrus_confidence)
    {
        return PixelQuality::Cloud;
    }
    if qa_pixel & CLOUD_SHADOW != 0 || confidence_policy.rejects(shadow_confidence) {
        return PixelQuality::CloudShadow;
    }
    if qa_pixel & SNOW != 0 || confidence_policy.rejects(snow_confidence) {
        return PixelQuality::SnowOrIce;
    }

    if qa_radsat & bands.required_saturation_mask() != 0 || qa_radsat & TERRAIN_OCCLUSION != 0 {
        return PixelQuality::OtherInvalid;
    }

    PixelQuality::Clear
}

/// Convert a raw Landsat Collection 2 Level-2 surface-reflectance DN to the
/// documented scaled value. DN=0 is the product fill value and returns None.
/// Apply QA_PIXEL/QA_RADSAT masking separately; scaling does not make a pixel
/// scientifically valid, and negative scaled reflectance is not clamped here.
pub fn scale_landsat_c2_surface_reflectance(dn: u16) -> Option<f64> {
    if dn == 0 {
        return None;
    }
    let reflectance = f64::from(dn) * LANDSAT_C2_SR_SCALE_FACTOR + LANDSAT_C2_SR_ADD_OFFSET;
    reflectance.is_finite().then_some(reflectance)
}

/// Decode Sentinel-2 Level-2A Scene Classification Layer (SCL).
///
/// The data-mask argument must come from the product's data/no-data mask if
/// that asset is available. Classes 2 (dark area), 7 (unclassified), and 1
/// (saturated/defective) are excluded conservatively. Class 6 (water) is
/// retained so water-index values can still be computed over water pixels; it
/// is not accepted as independent validation of the index result.
pub fn classify_sentinel2_l2a_scl(scl: Option<u8>, data_mask_is_valid: bool) -> PixelQuality {
    if !data_mask_is_valid {
        return PixelQuality::NoData;
    }

    match scl {
        None | Some(0) => PixelQuality::NoData,
        Some(1 | 2 | 7) => PixelQuality::OtherInvalid,
        Some(3) => PixelQuality::CloudShadow,
        Some(8 | 9 | 10) => PixelQuality::Cloud,
        Some(11) => PixelQuality::SnowOrIce,
        Some(4 | 5 | 6) => PixelQuality::Clear,
        Some(_) => PixelQuality::OtherInvalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_scl_masks_invalid_shadow_cloud_and_snow_classes() {
        assert_eq!(classify_sentinel2_l2a_scl(None, true), PixelQuality::NoData);
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(0), true),
            PixelQuality::NoData
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(1), true),
            PixelQuality::OtherInvalid
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(2), true),
            PixelQuality::OtherInvalid
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(3), true),
            PixelQuality::CloudShadow
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(7), true),
            PixelQuality::OtherInvalid
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(8), true),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(9), true),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(10), true),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(11), true),
            PixelQuality::SnowOrIce
        );
    }

    #[test]
    fn sentinel_scl_retains_surface_classes_and_honors_data_mask() {
        for class in [4, 5, 6] {
            assert_eq!(
                classify_sentinel2_l2a_scl(Some(class), true),
                PixelQuality::Clear
            );
        }
        assert_eq!(
            classify_sentinel2_l2a_scl(Some(6), false),
            PixelQuality::NoData
        );
    }

    #[test]
    fn landsat_fill_cloud_shadow_and_snow_bits_are_masked() {
        let bands = Landsat8WaterBandSet::NdwiGreenNir;
        let policy = Landsat8ConfidencePolicy::RejectMediumOrHigh;
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 0, 0, bands, policy),
            PixelQuality::NoData
        );
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 1, 0, bands, policy),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 2, 0, bands, policy),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 3, 0, bands, policy),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 4, 0, bands, policy),
            PixelQuality::CloudShadow
        );
        assert_eq!(
            classify_landsat8_c2_pixel(1 << 5, 0, bands, policy),
            PixelQuality::SnowOrIce
        );
    }

    #[test]
    fn landsat_water_flag_is_not_mistaken_for_invalid_data() {
        assert_eq!(
            classify_landsat8_c2_pixel(
                1 << 7,
                0,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::RejectMediumOrHigh
            ),
            PixelQuality::Clear
        );
    }

    #[test]
    fn landsat_rejects_medium_confidence_and_can_configure_high_only() {
        let medium_cloud = 2 << 8;
        assert_eq!(
            classify_landsat8_c2_pixel(
                medium_cloud,
                0,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::RejectMediumOrHigh
            ),
            PixelQuality::Cloud
        );
        assert_eq!(
            classify_landsat8_c2_pixel(
                medium_cloud,
                0,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::RejectHighOnly
            ),
            PixelQuality::Clear
        );
        let high_shadow = 3 << 10;
        assert_eq!(
            classify_landsat8_c2_pixel(
                high_shadow,
                0,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::RejectMediumOrHigh
            ),
            PixelQuality::CloudShadow
        );
    }

    #[test]
    fn landsat_rejects_saturation_only_in_required_bands() {
        assert_eq!(
            classify_landsat8_c2_pixel(
                0,
                1 << 2,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::default()
            ),
            PixelQuality::OtherInvalid
        );
        // Band 6 is not used by NDWI, so its saturation does not mask NDWI.
        assert_eq!(
            classify_landsat8_c2_pixel(
                0,
                1 << 5,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::default()
            ),
            PixelQuality::Clear
        );
        // Band 6 is required by MNDWI and therefore must be masked.
        assert_eq!(
            classify_landsat8_c2_pixel(
                0,
                1 << 5,
                Landsat8WaterBandSet::MndwiGreenSwir1,
                Landsat8ConfidencePolicy::default()
            ),
            PixelQuality::OtherInvalid
        );
    }

    #[test]
    fn landsat_terrain_occlusion_is_masked() {
        assert_eq!(
            classify_landsat8_c2_pixel(
                0,
                1 << 11,
                Landsat8WaterBandSet::NdwiGreenNir,
                Landsat8ConfidencePolicy::default()
            ),
            PixelQuality::OtherInvalid
        );
    }

    #[test]
    fn landsat_sr_scaling_handles_fill_and_matches_usgs_example() {
        assert_eq!(scale_landsat_c2_surface_reflectance(0), None);
        let reflectance = scale_landsat_c2_surface_reflectance(18_639).unwrap();
        assert!((reflectance - 0.312_572_5).abs() < 1e-12);
    }
}
