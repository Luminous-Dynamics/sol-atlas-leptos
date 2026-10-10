// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Safe URL construction for explicitly configured NASA GIBS Web Mercator tiles.
//!
//! This adapter only builds a tile URL from a configured layer, matrix set,
//! time and validated tile address. It does not fetch capabilities or images,
//! and it does not establish layer availability, source accuracy, or
//! redistribution permissions. Configurations should be populated from the
//! selected layer's current GIBS capabilities metadata and usage terms.

use crate::tiles::{MAX_TILE_ZOOM, TileCoordinate, TileError};
use std::fmt;

pub const NASA_GIBS_ATTRIBUTION: &str = "NASA Global Imagery Browse Services (GIBS)";
pub const NASA_GIBS_ACCESS_DOCUMENTATION: &str =
    "https://nasa-gibs.github.io/gibs-api-docs/access-basics/";
const NASA_GIBS_EPSG3857_BASE: &str = "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileImageFormat {
    Png,
    Jpeg,
}

impl TileImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }

    pub fn media_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GibsUrlError {
    InvalidLayerIdentifier,
    InvalidTileMatrixSet,
    InvalidTimeDimension,
    InvalidMaximumZoom(u8),
    ZoomExceedsLayerMaximum { requested: u8, maximum: u8 },
    Tile(TileError),
}

impl fmt::Display for GibsUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayerIdentifier => write!(f, "GIBS layer identifier is not path-safe"),
            Self::InvalidTileMatrixSet => write!(f, "GIBS tile matrix set identifier is not path-safe"),
            Self::InvalidTimeDimension => write!(f, "GIBS time dimension must be 'default', a YYYY-MM-DD date, or an RFC3339 UTC timestamp"),
            Self::InvalidMaximumZoom(zoom) => write!(f, "GIBS maximum zoom {zoom} exceeds {MAX_TILE_ZOOM}"),
            Self::ZoomExceedsLayerMaximum { requested, maximum } => {
                write!(f, "tile zoom {requested} exceeds this GIBS layer's configured maximum {maximum}")
            }
            Self::Tile(error) => write!(f, "invalid tile address: {error}"),
        }
    }
}

impl std::error::Error for GibsUrlError {}

impl From<TileError> for GibsUrlError {
    fn from(error: TileError) -> Self {
        Self::Tile(error)
    }
}

/// A configured GIBS layer in the EPSG:3857 REST XYZ-compatible interface.
///
/// The matrix-set and maximum zoom must come from the selected layer's service
/// capabilities. The helper is intentionally not a layer catalog: it cannot
/// infer supported dates, coverage, or maximum zoom from a layer identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GibsWebMercatorSource {
    layer_identifier: String,
    tile_matrix_set: String,
    maximum_zoom: u8,
    time_dimension: String,
    format: TileImageFormat,
}

impl GibsWebMercatorSource {
    pub fn new(
        layer_identifier: impl Into<String>,
        tile_matrix_set: impl Into<String>,
        maximum_zoom: u8,
        time_dimension: impl Into<String>,
        format: TileImageFormat,
    ) -> Result<Self, GibsUrlError> {
        let layer_identifier = layer_identifier.into();
        let tile_matrix_set = tile_matrix_set.into();
        let time_dimension = time_dimension.into();

        if !is_path_identifier(&layer_identifier) {
            return Err(GibsUrlError::InvalidLayerIdentifier);
        }
        if !is_path_identifier(&tile_matrix_set) {
            return Err(GibsUrlError::InvalidTileMatrixSet);
        }
        if maximum_zoom > MAX_TILE_ZOOM {
            return Err(GibsUrlError::InvalidMaximumZoom(maximum_zoom));
        }
        if !is_supported_time_dimension(&time_dimension) {
            return Err(GibsUrlError::InvalidTimeDimension);
        }

        Ok(Self {
            layer_identifier,
            tile_matrix_set,
            maximum_zoom,
            time_dimension,
            format,
        })
    }

    pub fn layer_identifier(&self) -> &str {
        &self.layer_identifier
    }

    pub fn tile_matrix_set(&self) -> &str {
        &self.tile_matrix_set
    }

    pub fn maximum_zoom(&self) -> u8 {
        self.maximum_zoom
    }

    pub fn time_dimension(&self) -> &str {
        &self.time_dimension
    }

    pub fn format(&self) -> TileImageFormat {
        self.format
    }

    pub fn attribution(&self) -> &'static str {
        NASA_GIBS_ATTRIBUTION
    }

    pub fn access_documentation(&self) -> &'static str {
        NASA_GIBS_ACCESS_DOCUMENTATION
    }

    /// Build a public GIBS REST XYZ-style URL for this tile.
    ///
    /// The caller owns the actual network request, caching, rate/concurrency
    /// limits, stale-response handling, and display/attribution lifecycle.
    pub fn tile_url(&self, tile: TileCoordinate) -> Result<String, GibsUrlError> {
        if tile.zoom() > self.maximum_zoom {
            return Err(GibsUrlError::ZoomExceedsLayerMaximum {
                requested: tile.zoom(),
                maximum: self.maximum_zoom,
            });
        }

        Ok(format!(
            "{}/{}/default/{}/{}/{}/{}/{}.{}",
            NASA_GIBS_EPSG3857_BASE,
            self.layer_identifier,
            self.time_dimension,
            self.tile_matrix_set,
            tile.zoom(),
            tile.y(),
            tile.x(),
            self.format.extension()
        ))
    }
}

fn is_path_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn is_supported_time_dimension(value: &str) -> bool {
    if value == "default" {
        return true;
    }
    let bytes = value.as_bytes();
    if bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return true;
    }
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_deterministic_gibs_url_for_selected_matrix_set() {
        let source = GibsWebMercatorSource::new(
            "MODIS_Terra_Aerosol",
            "GoogleMapsCompatible_Level6",
            6,
            "2014-04-09",
            TileImageFormat::Png,
        )
        .unwrap();
        let tile = TileCoordinate::new(3, 2, 1).unwrap();
        assert_eq!(
            source.tile_url(tile).unwrap(),
            "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/MODIS_Terra_Aerosol/default/2014-04-09/GoogleMapsCompatible_Level6/3/1/2.png"
        );
        assert_eq!(source.attribution(), "NASA Global Imagery Browse Services (GIBS)");
        assert_eq!(source.format().media_type(), "image/png");
    }

    #[test]
    fn supports_default_and_utc_timestamp_time_dimensions() {
        let default = GibsWebMercatorSource::new(
            "MODIS_Terra_Aerosol",
            "GoogleMapsCompatible_Level6",
            6,
            "default",
            TileImageFormat::Jpeg,
        )
        .unwrap();
        assert!(default.tile_url(TileCoordinate::new(0, 0, 0).unwrap()).unwrap().ends_with("/0/0/0.jpg"));

        assert!(GibsWebMercatorSource::new(
            "layer",
            "matrix",
            5,
            "2026-10-10T12:00:00Z",
            TileImageFormat::Png,
        ).is_ok());
        assert!(GibsWebMercatorSource::new(
            "layer",
            "matrix",
            5,
            "2026-10-10/2026-10-11",
            TileImageFormat::Png,
        ).is_err());
    }

    #[test]
    fn rejects_path_injection_and_layer_zoom_mismatch() {
        assert_eq!(
            GibsWebMercatorSource::new("../other", "matrix", 6, "default", TileImageFormat::Png),
            Err(GibsUrlError::InvalidLayerIdentifier)
        );
        assert_eq!(
            GibsWebMercatorSource::new("layer", "../matrix", 6, "default", TileImageFormat::Png),
            Err(GibsUrlError::InvalidTileMatrixSet)
        );

        let source = GibsWebMercatorSource::new(
            "layer",
            "GoogleMapsCompatible_Level2",
            2,
            "default",
            TileImageFormat::Png,
        )
        .unwrap();
        assert_eq!(
            source.tile_url(TileCoordinate::new(3, 2, 1).unwrap()),
            Err(GibsUrlError::ZoomExceedsLayerMaximum { requested: 3, maximum: 2 })
        );
    }
}
