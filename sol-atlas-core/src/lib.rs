// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).
//!
//! All math uses raw [f32; N] arrays to avoid dependency collisions
//! between rendering backends (web-sys, glam/nalgebra).

pub mod aesthetics;
pub mod ags_preflight;
pub mod bgs_sobi;
pub mod confluence;
pub mod constants;
pub mod data;
pub mod earth_observability;
pub mod earth_observability_synthetic;
pub mod evidence_revalidation;
pub mod field_activity_admission;
pub mod investigation_tournament;
pub mod economics;
pub mod energy_trading;
pub mod geo;
pub mod lod;
pub mod mycelix_flows;
// pub mod relativity_viz;
pub mod geometry;
pub mod math;
pub mod network_hypotheses;
pub mod picking;
pub mod project_split;
pub mod sensor_evsi_bridge;
pub mod sensor_observability;
pub mod simulation;
pub mod solar_system;
pub mod timeline;
pub mod types;
pub mod utility_inference;
pub mod utility_reconstruction_metrics;
pub mod visual_validation;

pub use types::*;
