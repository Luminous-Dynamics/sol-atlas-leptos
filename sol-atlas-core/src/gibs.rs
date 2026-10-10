// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Safe URL construction for explicitly configured NASA GIBS Web Mercator tiles.
//!
//! This adapter only builds a tile URL from a configured layer, matrix set,
//! time and validated tile address. It does not fetch capabilities or images,
//! and it does not establish layer availability, source accuracy, or
//! redistribution permissions. Configurations should be populated from the
//! selected layer's current GIBS capabilities metadata and usage terms.

use crate::tiles::{tiles_for_bounds, MAX_TILE_ZOOM, TileCoordinate, TileError};
use std::fmt;

pub const NASA_GIBS_SERVICE_ACKNOWLEDGEMENT: &str = "NASA Global Imagery Browse Services (GIBS)";
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
    InvalidLayerAttribution,
    InvalidMaximumZoom(u8),
    ZoomExceedsLayerMaximum { requested: u8, maximum: u8 },
    Tile(TileError),
}

impl fmt::Display for GibsUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLayerIdentifier => write!(f, "GIBS layer identifier is not path-safe"),
            Self::InvalidTileMatrixSet => {
                write!(f, "GIBS tile matrix set identifier is not path-safe")
            },
            Self::InvalidTimeDimension => write!(
                f,
                "GIBS time dimension must be 'default', a YYYY-MM-DD date, or an RFC3339 UTC timestamp"
            ),
            Self::InvalidLayerAttribution => {
                write!(f, "GIBS layer attribution must be present and non-empty")
            },
            Self::InvalidMaximumZoom(zoom) => {
                write!(f, "GIBS maximum zoom {zoom} exceeds {MAX_TILE_ZOOM}")
            },
            Self::ZoomExceedsLayerMaximum { requested, maximum } => {
                write!(
                    f,
                    "tile zoom {requested} exceeds this GIBS layer's configured maximum {maximum}"
                )
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
    layer_attribution: String,
}

impl GibsWebMercatorSource {
    pub fn new(
        layer_identifier: impl Into<String>,
        tile_matrix_set: impl Into<String>,
        maximum_zoom: u8,
        time_dimension: impl Into<String>,
        format: TileImageFormat,
        layer_attribution: impl Into<String>,
    ) -> Result<Self, GibsUrlError> {
        let layer_identifier = layer_identifier.into();
        let tile_matrix_set = tile_matrix_set.into();
        let time_dimension = time_dimension.into();
        let layer_attribution = layer_attribution.into();

        if !is_path_identifier(&layer_identifier) {
            return Err(GibsUrlError::InvalidLayerIdentifier);
        }
        if !is_path_identifier(&tile_matrix_set) {
            return Err(GibsUrlError::InvalidTileMatrixSet);
        }
        if maximum_zoom > MAX_TILE_ZOOM {
            return Err(GibsUrlError::InvalidMaximumZoom(maximum_zoom));
        }
        if layer_attribution.trim().is_empty() {
            return Err(GibsUrlError::InvalidLayerAttribution);
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
            layer_attribution,
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

    /// Layer-specific attribution taken from current provider metadata.
    pub fn attribution(&self) -> &str {
        &self.layer_attribution
    }

    /// Service acknowledgement is distinct from a layer's source credits.
    pub fn service_acknowledgement(&self) -> &'static str {
        NASA_GIBS_SERVICE_ACKNOWLEDGEMENT
    }

    pub fn access_documentation(&self) -> &'static str {
        NASA_GIBS_ACCESS_DOCUMENTATION
    }

    /// Plan a bounded, deterministic URL list for every tile intersecting a
    /// WGS84 bbox. Results are row-major XYZ order, including across the
    /// antimeridian. The budget is enforced before URL allocation.
    ///
    /// This is a planning primitive only; it does not dispatch HTTP requests.
    pub fn tile_urls_for_bounds(
        &self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        zoom: u8,
        max_tiles: usize,
    ) -> Result<Vec<String>, GibsUrlError> {
        if zoom > self.maximum_zoom {
            return Err(GibsUrlError::ZoomExceedsLayerMaximum {
                requested: zoom,
                maximum: self.maximum_zoom,
            });
        }

        tiles_for_bounds(west, south, east, north, zoom, max_tiles)?
            .into_iter()
            .map(|tile| self.tile_url(tile))
            .collect()
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
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn is_supported_time_dimension(value: &str) -> bool {
    if value == "default" {
        return true;
    }

    let bytes = value.as_bytes();
    if bytes.len() == 10 {
        return is_valid_utc_date(bytes);
    }

    if bytes.len() != 20
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }

    if !is_valid_utc_date(&bytes[..10]) {
        return false;
    }

    let Some(hour) = parse_digits(&bytes[11..13]) else {
        return false;
    };
    let Some(minute) = parse_digits(&bytes[14..16]) else {
        return false;
    };
    let Some(second) = parse_digits(&bytes[17..19]) else {
        return false;
    };

    hour <= 23 && minute <= 59 && second <= 60
}

fn is_valid_utc_date(bytes: &[u8]) -> bool {
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }

    let Some(year) = parse_digits(&bytes[0..4]) else {
        return false;
    };
    let Some(month) = parse_digits(&bytes[5..7]) else {
        return false;
    };
    let Some(day) = parse_digits(&bytes[8..10]) else {
        return false;
    };

    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }

    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return false,
    };

    (1..=days_in_month).contains(&day)
}

fn parse_digits(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    bytes.iter().try_fold(0_u32, |value, digit| {
        value
            .checked_mul(10)?
            .checked_add(u32::from(*digit - b'0'))
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
            5, // six matrix levels are numbered 0 through 5
            "2014-04-09",
            TileImageFormat::Png,
            "Fixture layer attribution",
        )
        .unwrap();
        let tile = TileCoordinate::new(3, 2, 1).unwrap();
        assert_eq!(
            source.tile_url(tile).unwrap(),
            "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/MODIS_Terra_Aerosol/default/2014-04-09/GoogleMapsCompatible_Level6/3/1/2.png"
        );
        assert_eq!(source.attribution(), "Fixture layer attribution");
        assert_eq!(
            source.service_acknowledgement(),
            "NASA Global Imagery Browse Services (GIBS)"
        );
        assert_eq!(source.format().media_type(), "image/png");
    }

    #[test]
    fn supports_default_and_utc_timestamp_time_dimensions() {
        let default = GibsWebMercatorSource::new(
            "MODIS_Terra_Aerosol",
            "GoogleMapsCompatible_Level6",
            5, // six matrix levels are numbered 0 through 5
            "default",
            TileImageFormat::Jpeg,
            "Fixture layer attribution",
        )
        .unwrap();
        assert!(
            default
                .tile_url(TileCoordinate::new(0, 0, 0).unwrap())
                .unwrap()
                .ends_with("/0/0/0.jpg")
        );

        assert!(GibsWebMercatorSource::new(
            "layer",
            "matrix",
            5,
            "2026-10-10T12:00:00Z",
            TileImageFormat::Png,
            "Fixture layer attribution",
        ).is_ok());
        assert!(GibsWebMercatorSource::new(
            "layer",
            "matrix",
            5,
            "2026-10-10/2026-10-11",
            TileImageFormat::Png,
            "Fixture layer attribution",
        ).is_err());
    }

    #[test]
    fn plans_bounded_deterministic_urls_for_a_geographic_extent() {
        let source = GibsWebMercatorSource::new(
            "Fixture_Global_Browse",
            "GoogleMapsCompatible_Level6",
            5,
            "2026-10-10",
            TileImageFormat::Png,
            "Fixture source attribution",
        )
        .unwrap();

        let urls = source
            .tile_urls_for_bounds(-90.0, -10.0, 0.0, 10.0, 1, 4)
            .unwrap();
        assert_eq!(urls.len(), 4);
        assert_eq!(
            urls,
            vec![
                "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/Fixture_Global_Browse/default/2026-10-10/GoogleMapsCompatible_Level6/1/0/0.png",
                "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/Fixture_Global_Browse/default/2026-10-10/GoogleMapsCompatible_Level6/1/0/1.png",
                "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/Fixture_Global_Browse/default/2026-10-10/GoogleMapsCompatible_Level6/1/1/0.png",
                "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/Fixture_Global_Browse/default/2026-10-10/GoogleMapsCompatible_Level6/1/1/1.png"
            ]
        );

        assert!(matches!(
            source.tile_urls_for_bounds(-180.0, -85.0, 180.0, 85.0, 3, 4),
            Err(GibsUrlError::Tile(TileError::TileBudgetExceeded { .. }))
        ));
        assert!(matches!(
            source.tile_urls_for_bounds(-90.0, -10.0, 0.0, 10.0, 6, 100),
            Err(GibsUrlError::ZoomExceedsLayerMaximum { requested: 6, maximum: 5 })
        ));
    }

    #[test]
    fn validates_real_calendar_dates_and_utc_clock_ranges() {
        let make = |time: &str| {
            GibsWebMercatorSource::new(
                "layer",
                "matrix",
                6,
                time,
                TileImageFormat::Png,
                "Fixture layer attribution",
            )
        };

        assert!(make("2024-02-29").is_ok());
        assert!(make("2026-02-29").is_err());
        assert!(make("2026-13-01").is_err());
        assert!(make("2026-04-31").is_err());
        assert!(make("2026-10-10T23:59:59Z").is_ok());
        assert!(make("2026-10-10T24:00:00Z").is_err());
        assert!(make("2026-10-10T12:60:00Z").is_err());
        assert!(make("2026-10-10T12:00:61Z").is_err());
    }

    #[test]
    fn rejects_path_injection_and_layer_zoom_mismatch() {
        assert_eq!(
            GibsWebMercatorSource::new(
                "../other",
                "matrix",
                6,
                "default",
                TileImageFormat::Png,
                "Fixture layer attribution"
            ),
            Err(GibsUrlError::InvalidLayerIdentifier)
        );
        assert_eq!(
            GibsWebMercatorSource::new(
                "..",
                "matrix",
                6,
                "default",
                TileImageFormat::Png,
                "Fixture layer attribution"
            ),
            Err(GibsUrlError::InvalidLayerIdentifier)
        );
        assert_eq!(
            GibsWebMercatorSource::new(
                "layer",
                "../matrix",
                6,
                "default",
                TileImageFormat::Png,
                "Fixture layer attribution"
            ),
            Err(GibsUrlError::InvalidTileMatrixSet)
        );

        let source = GibsWebMercatorSource::new(
            "layer",
            "GoogleMapsCompatible_Level2",
            1, // two matrix levels are numbered 0 through 1
            "default",
            TileImageFormat::Png,
            "Fixture layer attribution",
        )
        .unwrap();
        assert_eq!(
            source.tile_url(TileCoordinate::new(3, 2, 1).unwrap()),
            Err(GibsUrlError::ZoomExceedsLayerMaximum { requested: 3, maximum: 1 })
        );
    }
}
