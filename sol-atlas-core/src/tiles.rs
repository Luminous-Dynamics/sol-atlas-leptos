// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic Web Mercator XYZ/TMS tile addressing.
//!
//! This module is deliberately network- and renderer-neutral. It calculates
//! tile identities and geographic extents; it does not fetch, cache, or render
//! tiles. Tile bounds use longitude/latitude in WGS84 degrees. Web Mercator
//! cannot represent the polar caps, so point lookup offers both explicit
//! rejection and an explicitly named clamping variant.

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Deliberate upper bound for tile zoom in this implementation.
///
/// The limit bounds address arithmetic and prevents the catalog/tile-selection
/// layer from implicitly authorizing arbitrarily deep requests.
pub const MAX_TILE_ZOOM: u8 = 30;

/// Maximum latitude representable by the spherical Web Mercator projection.
pub const MAX_MERCATOR_LATITUDE: f64 = 85.051_128_779_806_6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileError {
    InvalidZoom(u8),
    NonFiniteCoordinate,
    LongitudeOutOfRange,
    LatitudeOutOfRange,
    InvalidBounds,
    TileIndexOutOfRange,
    TileBudgetExceeded { requested: u64, maximum: usize },
}

impl fmt::Display for TileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidZoom(zoom) => write!(f, "tile zoom {zoom} exceeds maximum {MAX_TILE_ZOOM}"),
            Self::NonFiniteCoordinate => write!(f, "geographic coordinates must be finite"),
            Self::LongitudeOutOfRange => write!(f, "longitude must be within [-180, 180] for bounds"),
            Self::LatitudeOutOfRange => write!(f, "latitude must be within [-90, 90]"),
            Self::InvalidBounds => write!(f, "bounds must have south < north and non-zero longitude width"),
            Self::TileIndexOutOfRange => write!(f, "tile index is outside the selected zoom level"),
            Self::TileBudgetExceeded { requested, maximum } => {
                write!(f, "tile selection requests {requested} tiles, exceeding budget {maximum}")
            }
        }
    }
}

impl std::error::Error for TileError {}

/// Scheme describes how a tile row is numbered, not how tile pixels are projected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileRowScheme {
    /// Slippy-map convention: row 0 is at the north.
    Xyz,
    /// TMS convention: row 0 is at the south.
    Tms,
}

/// Stable address for one square tile in the Web Mercator pyramid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TileCoordinate {
    zoom: u8,
    x: u32,
    y: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TileCoordinateWire {
    zoom: u8,
    x: u32,
    y: u32,
}

impl TileCoordinate {
    pub fn new(zoom: u8, x: u32, y: u32) -> Result<Self, TileError> {
        validate_zoom(zoom)?;
        let count = tile_count(zoom);
        if u64::from(x) >= count || u64::from(y) >= count {
            return Err(TileError::TileIndexOutOfRange);
        }
        Ok(Self { zoom, x, y })
    }

    /// Interpret an XYZ row or a TMS south-origin row.
    pub fn from_row_scheme(
        zoom: u8,
        x: u32,
        row: u32,
        scheme: TileRowScheme,
    ) -> Result<Self, TileError> {
        validate_zoom(zoom)?;
        let count = tile_count(zoom);
        if u64::from(x) >= count || u64::from(row) >= count {
            return Err(TileError::TileIndexOutOfRange);
        }
        let y = match scheme {
            TileRowScheme::Xyz => row,
            TileRowScheme::Tms => (count - 1 - u64::from(row)) as u32,
        };
        Self::new(zoom, x, y)
    }

    /// Point lookup that rejects valid geographic latitudes outside Web Mercator.
    ///
    /// Longitude outside [-180, 180] is wrapped across the antimeridian. Exactly
    /// +180 maps to the easternmost tile, while -180 maps to the westernmost.
    pub fn from_lon_lat(lon: f64, lat: f64, zoom: u8) -> Result<Self, TileError> {
        validate_zoom(zoom)?;
        if !lon.is_finite() || !lat.is_finite() {
            return Err(TileError::NonFiniteCoordinate);
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(TileError::LatitudeOutOfRange);
        }
        if !(-MAX_MERCATOR_LATITUDE..=MAX_MERCATOR_LATITUDE).contains(&lat) {
            return Err(TileError::LatitudeOutOfRange);
        }
        Self::from_lon_lat_mercator(wrap_longitude(lon), lat, zoom)
    }

    /// Point lookup that clamps valid geographic latitudes to Web Mercator's limit.
    ///
    /// Use this only when mapping polar locations onto the edge of the projected
    /// map is intended. It does not imply that Web Mercator represents the pole.
    pub fn from_lon_lat_clamped(
        lon: f64,
        lat: f64,
        zoom: u8,
    ) -> Result<Self, TileError> {
        validate_zoom(zoom)?;
        if !lon.is_finite() || !lat.is_finite() {
            return Err(TileError::NonFiniteCoordinate);
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(TileError::LatitudeOutOfRange);
        }
        let clamped_lat = lat.clamp(-MAX_MERCATOR_LATITUDE, MAX_MERCATOR_LATITUDE);
        Self::from_lon_lat_mercator(wrap_longitude(lon), clamped_lat, zoom)
    }

    fn from_lon_lat_mercator(lon: f64, lat: f64, zoom: u8) -> Result<Self, TileError> {
        let count = tile_count(zoom) as f64;
        let x = (((lon + 180.0) / 360.0) * count).floor().clamp(0.0, count - 1.0) as u32;
        let latitude_radians = lat.to_radians();
        let y_fraction =
            (1.0 - latitude_radians.tan().asinh() / std::f64::consts::PI) / 2.0;
        let y = (y_fraction * count).floor().clamp(0.0, count - 1.0) as u32;
        Self::new(zoom, x, y)
    }

    pub fn zoom(self) -> u8 {
        self.zoom
    }

    pub fn x(self) -> u32 {
        self.x
    }

    /// Normalized XYZ row, where row 0 is north.
    pub fn y(self) -> u32 {
        self.y
    }

    pub fn row(self, scheme: TileRowScheme) -> u32 {
        match scheme {
            TileRowScheme::Xyz => self.y,
            TileRowScheme::Tms => (tile_count(self.zoom) - 1 - u64::from(self.y)) as u32,
        }
    }

    /// Geographic footprint of the tile in WGS84 longitude/latitude degrees.
    /// At polar rows, north/south are the Web Mercator limit, not +/-90 degrees.
    pub fn extent(self) -> TileExtent {
        let count = tile_count(self.zoom) as f64;
        let west = f64::from(self.x) / count * 360.0 - 180.0;
        let east = (f64::from(self.x) + 1.0) / count * 360.0 - 180.0;
        let north = tile_y_to_latitude(f64::from(self.y), count);
        let south = tile_y_to_latitude(f64::from(self.y) + 1.0, count);
        TileExtent { west, south, east, north }
    }
}

impl<'de> Deserialize<'de> for TileCoordinate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TileCoordinateWire::deserialize(deserializer)?;
        Self::new(wire.zoom, wire.x, wire.y).map_err(serde::de::Error::custom)
    }
}

/// Geographic bounding box for one tile or selected region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileExtent {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

/// Return tiles intersecting a geographic bounding box in deterministic row-major order.
///
/// Bounds are WGS84 degrees. A box with west > east crosses the antimeridian.
/// Bounds may include the poles; its north/south latitude is clamped to Web
/// Mercator's maximum. The result is rejected before allocation if it would
/// exceed max_tiles. Exact shared tile edges may conservatively include both
/// adjacent tiles, preventing visible cracks when requests use rounded bounds.
pub fn tiles_for_bounds(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    zoom: u8,
    max_tiles: usize,
) -> Result<Vec<TileCoordinate>, TileError> {
    validate_zoom(zoom)?;
    if [west, south, east, north].iter().any(|v| !v.is_finite()) {
        return Err(TileError::NonFiniteCoordinate);
    }
    if !(-180.0..=180.0).contains(&west) || !(-180.0..=180.0).contains(&east) {
        return Err(TileError::LongitudeOutOfRange);
    }
    if !(-90.0..=90.0).contains(&south) || !(-90.0..=90.0).contains(&north) {
        return Err(TileError::LatitudeOutOfRange);
    }
    if south >= north || west == east || (west == 180.0 && east == -180.0) {
        return Err(TileError::InvalidBounds);
    }

    let count = tile_count(zoom);
    let north_clamped = north.clamp(-MAX_MERCATOR_LATITUDE, MAX_MERCATOR_LATITUDE);
    let south_clamped = south.clamp(-MAX_MERCATOR_LATITUDE, MAX_MERCATOR_LATITUDE);
    let y_top = latitude_to_tile_y(north_clamped, count);
    let y_bottom = latitude_to_tile_y(south_clamped, count);
    let rows = u64::from(y_bottom) - u64::from(y_top) + 1;

    let x_west = longitude_to_tile_x(west, count);
    let x_east = longitude_to_tile_x(east, count);
    let x_columns = if west <= east {
        u64::from(x_east) - u64::from(x_west) + 1
    } else {
        (count - u64::from(x_west)) + (u64::from(x_east) + 1)
    };
    let requested = rows * x_columns;
    if requested > max_tiles as u64 {
        return Err(TileError::TileBudgetExceeded { requested, maximum: max_tiles });
    }

    let mut columns = Vec::with_capacity((x_columns as usize).min(max_tiles));
    if west <= east {
        columns.extend(x_west..=x_east);
    } else {
        columns.extend(x_west..=(count as u32 - 1));
        columns.extend(0..=x_east);
    }

    let mut result = Vec::with_capacity(requested as usize);
    for y in y_top..=y_bottom {
        for x in &columns {
            result.push(TileCoordinate::new(zoom, *x, y)?);
        }
    }
    Ok(result)
}

fn validate_zoom(zoom: u8) -> Result<(), TileError> {
    if zoom > MAX_TILE_ZOOM {
        Err(TileError::InvalidZoom(zoom))
    } else {
        Ok(())
    }
}

fn tile_count(zoom: u8) -> u64 {
    1_u64 << zoom
}

fn wrap_longitude(lon: f64) -> f64 {
    if (-180.0..=180.0).contains(&lon) {
        lon
    } else {
        (lon + 180.0).rem_euclid(360.0) - 180.0
    }
}

fn longitude_to_tile_x(lon: f64, count: u64) -> u32 {
    let normalized = wrap_longitude(lon);
    (((normalized + 180.0) / 360.0) * count as f64)
        .floor()
        .clamp(0.0, count as f64 - 1.0) as u32
}

fn latitude_to_tile_y(lat: f64, count: u64) -> u32 {
    let latitude_radians = lat.to_radians();
    let fraction =
        (1.0 - latitude_radians.tan().asinh() / std::f64::consts::PI) / 2.0;
    (fraction * count as f64)
        .floor()
        .clamp(0.0, count as f64 - 1.0) as u32
}

fn tile_y_to_latitude(y: f64, count: f64) -> f64 {
    (std::f64::consts::PI * (1.0 - 2.0 * y / count))
        .sinh()
        .atan()
        .to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_zero_covers_world_inside_web_mercator_limits() {
        let tile = TileCoordinate::new(0, 0, 0).unwrap();
        let extent = tile.extent();
        assert_eq!(extent.west, -180.0);
        assert_eq!(extent.east, 180.0);
        assert!((extent.north - MAX_MERCATOR_LATITUDE).abs() < 1e-9);
        assert!((extent.south + MAX_MERCATOR_LATITUDE).abs() < 1e-9);
    }

    #[test]
    fn xyz_point_addressing_handles_edges_and_longitude_wrapping() {
        let west = TileCoordinate::from_lon_lat(-180.0, 0.0, 1).unwrap();
        let east = TileCoordinate::from_lon_lat(180.0, 0.0, 1).unwrap();
        assert_eq!((west.x(), west.y()), (0, 1));
        assert_eq!((east.x(), east.y()), (1, 1));

        let wrapped_east = TileCoordinate::from_lon_lat(181.0, 0.0, 2).unwrap();
        let wrapped_west = TileCoordinate::from_lon_lat(-181.0, 0.0, 2).unwrap();
        assert_eq!(wrapped_east.x(), 0);
        assert_eq!(wrapped_west.x(), 3);
    }

    #[test]
    fn tms_rows_are_explicitly_inverted_from_xyz() {
        let tile = TileCoordinate::new(3, 2, 1).unwrap();
        assert_eq!(tile.row(TileRowScheme::Xyz), 1);
        assert_eq!(tile.row(TileRowScheme::Tms), 6);
        assert_eq!(
            TileCoordinate::from_row_scheme(3, 2, 6, TileRowScheme::Tms).unwrap(),
            tile
        );
    }

    #[test]
    fn polar_latitudes_are_rejected_or_explicitly_clamped() {
        assert_eq!(
            TileCoordinate::from_lon_lat(0.0, 90.0, 3),
            Err(TileError::LatitudeOutOfRange)
        );
        assert_eq!(
            TileCoordinate::from_lon_lat_clamped(0.0, 90.0, 3).unwrap().y(),
            0
        );
        assert_eq!(
            TileCoordinate::from_lon_lat_clamped(0.0, -90.0, 3).unwrap().y(),
            7
        );
        assert_eq!(
            TileCoordinate::from_lon_lat(0.0, f64::NAN, 3),
            Err(TileError::NonFiniteCoordinate)
        );
    }

    #[test]
    fn bounds_selection_handles_antimeridian_in_row_major_order() {
        let tiles = tiles_for_bounds(170.0, -10.0, -170.0, 10.0, 2, 10).unwrap();
        let keys: Vec<_> = tiles.iter().map(|t| (t.y(), t.x())).collect();
        assert_eq!(keys, vec![(1, 0), (1, 3), (2, 0), (2, 3)]);
    }

    #[test]
    fn bounds_selection_clamps_poles_and_enforces_budget_before_allocation() {
        let polar = tiles_for_bounds(-10.0, -90.0, 10.0, 90.0, 3, 100).unwrap();
        assert!(polar.iter().any(|tile| tile.y() == 0));
        assert!(polar.iter().any(|tile| tile.y() == 7));

        assert_eq!(
            tiles_for_bounds(-180.0, -85.0, 180.0, 85.0, 3, 4),
            Err(TileError::TileBudgetExceeded { requested: 64, maximum: 4 })
        );
        assert_eq!(
            tiles_for_bounds(-180.0, -10.0, 180.0, 10.0, 31, 4),
            Err(TileError::InvalidZoom(31))
        );
    }

    #[test]
    fn invalid_coordinates_and_deserialization_fail_closed() {
        assert!(TileCoordinate::new(2, 4, 0).is_err());
        assert!(TileCoordinate::new(MAX_TILE_ZOOM + 1, 0, 0).is_err());
        assert!(TileCoordinate::from_lon_lat(0.0, 100.0, 3).is_err());
        assert!(TileCoordinate::from_lon_lat(f64::INFINITY, 0.0, 3).is_err());
        assert!(serde_json::from_str::<TileCoordinate>(r#"{"zoom":2,"x":4,"y":0}"#).is_err());
    }

    #[test]
    fn region_selection_is_deterministic() {
        let a = tiles_for_bounds(10.0, -5.0, 20.0, 5.0, 5, 100).unwrap();
        let b = tiles_for_bounds(10.0, -5.0, 20.0, 5.0, 5, 100).unwrap();
        assert_eq!(a, b);
        assert!(a.windows(2).all(|pair| {
            (pair[0].y(), pair[0].x()) <= (pair[1].y(), pair[1].x())
        }));
    }
}
