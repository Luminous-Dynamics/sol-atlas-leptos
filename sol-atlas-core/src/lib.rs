// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).
//!
//! All math uses raw `[f32; N]` arrays to avoid dependency collisions
//! between rendering backends (web-sys, glam/nalgebra).

pub mod aesthetics;
pub mod ai_compute_energy;
pub mod ai_compute_energy_frontier;
pub mod confluence;
pub mod constants;
pub mod data;
pub mod demand_response_metrics;
pub mod economics;
pub mod energy_trading;
pub mod energy_storage;
pub mod energy_pathway;
pub mod energy_ports;
pub mod geo;
pub mod lod;
pub mod mycelix_flows;
// pub mod relativity_viz;
pub mod geometry;
pub mod heat_pump;
pub mod thermal_exergy;
pub mod math;
pub mod picking;
pub mod simulation;
pub mod solar_system;
pub mod timeline;
pub mod types;
pub mod visual_validation;

pub use types::*;
