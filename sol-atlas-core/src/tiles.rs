// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validated XYZ/Web Mercator tile coordinates shared by map tile providers.

use std::fmt;

/// Deliberately conservative maximum zoom supported by the current renderer.
pub const MAX_TILE_ZOOM: u8 = 22;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileError {
    ZoomOutOfRange(u8),
    XOutOfRange { zoom: u8, x: u32 },
    YOutOfRange { zoom: u8, y: u32 },
}

impl fmt::Display for TileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZoomOutOfRange(zoom) => write!(f, "tile zoom {zoom} exceeds {MAX_TILE_ZOOM}"),
            Self::XOutOfRange { zoom, x } => write!(f, "tile x={x} is outside zoom {zoom}"),
            Self::YOutOfRange { zoom, y } => write!(f, "tile y={y} is outside zoom {zoom}"),
        }
    }
}

impl std::error::Error for TileError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoordinate {
    zoom: u8,
    x: u32,
    y: u32,
}

impl TileCoordinate {
    pub fn new(zoom: u8, x: u32, y: u32) -> Result<Self, TileError> {
        if zoom > MAX_TILE_ZOOM {
            return Err(TileError::ZoomOutOfRange(zoom));
        }
        let matrix_size = 1_u32 << zoom;
        if x >= matrix_size {
            return Err(TileError::XOutOfRange { zoom, x });
        }
        if y >= matrix_size {
            return Err(TileError::YOutOfRange { zoom, y });
        }
        Ok(Self { zoom, x, y })
    }

    pub fn zoom(self) -> u8 {
        self.zoom
    }

    pub fn x(self) -> u32 {
        self.x
    }

    pub fn y(self) -> u32 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_origin_at_zero_zoom() {
        let tile = TileCoordinate::new(0, 0, 0).unwrap();
        assert_eq!((tile.zoom(), tile.x(), tile.y()), (0, 0, 0));
    }

    #[test]
    fn rejects_coordinates_outside_matrix() {
        assert_eq!(
            TileCoordinate::new(2, 4, 0),
            Err(TileError::XOutOfRange { zoom: 2, x: 4 })
        );
        assert_eq!(
            TileCoordinate::new(2, 0, 4),
            Err(TileError::YOutOfRange { zoom: 2, y: 4 })
        );
    }

    #[test]
    fn rejects_zoom_above_supported_maximum() {
        assert_eq!(
            TileCoordinate::new(MAX_TILE_ZOOM + 1, 0, 0),
            Err(TileError::ZoomOutOfRange(MAX_TILE_ZOOM + 1))
        );
    }
}
