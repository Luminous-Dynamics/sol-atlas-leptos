// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-aware visualization helpers.
//!
//! Missing or invalid physical measurements must not be mapped to a plausible
//! category. This module provides neutral visual treatment for unknown values.

/// Neutral grey used when no valid physical EROI evidence is available.
pub const UNKNOWN_EROI_COLOR: [f32; 3] = [0.48, 0.50, 0.54];

/// Map a supplied physical EROI to its indicative tier color; unknown and invalid
/// values are deliberately unranked and use a neutral grey.
pub fn eroi_color_or_unknown(eroi: Option<f64>) -> [f32; 3] {
    match eroi {
        Some(value) if value.is_finite() && value > 0.0 => crate::economics::eroi_color(value),
        _ => UNKNOWN_EROI_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_invalid_eroi_is_not_given_an_indicative_tier_color() {
        assert_eq!(eroi_color_or_unknown(None), UNKNOWN_EROI_COLOR);
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(eroi_color_or_unknown(Some(invalid)), UNKNOWN_EROI_COLOR);
        }
    }

    #[test]
    fn valid_eroi_uses_the_existing_boundary_specific_tier_palette() {
        for value in [2.0, 4.0, 7.0, 15.0] {
            assert_eq!(
                eroi_color_or_unknown(Some(value)),
                crate::economics::eroi_color(value)
            );
        }
    }
}
