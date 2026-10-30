// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).
//!
//! All math uses raw [f32; N] arrays to avoid dependency collisions
//! between rendering backends (web-sys, glam/nalgebra).

pub mod aesthetics;
pub mod common_cause;
pub mod confluence;
pub mod constants;
pub mod data;
pub mod dependency_logic;
pub mod economics;
pub mod energy_trading;
pub mod geo;
pub mod geometry;
pub mod intervention;
pub mod lod;
pub mod math;
pub mod mycelix_flows;
pub mod picking;
pub mod service_equity;
pub mod service_function;
pub mod service_network;
pub mod service_network_intervention;
pub mod service_network_portfolio;
pub mod service_network_stress;
pub mod service_recovery;
pub mod service_shared_resource_allocation;
pub mod service_water;
pub mod service_water_energy_coupling;
pub mod service_water_energy_intervention;
pub mod service_water_energy_stress;
pub mod service_water_energy_stress_intervention;
pub mod service_water_trajectory;
pub mod service_water_trajectory_intervention;
pub mod telecom_power;
pub mod simulation;
pub mod solar_system;
pub mod spatial_economic;
pub mod spatial_graph;
pub mod timeline;
pub mod types;
pub mod visual_validation;

pub use types::*;
