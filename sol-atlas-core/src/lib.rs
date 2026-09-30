// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).

pub mod aesthetics;
pub mod confluence;
pub mod civilizational;
pub mod cultural_systems;
pub mod cultural_argumentation;
pub mod ontology_mapping;
pub mod ontology_context;
pub mod ontology_resolution;
pub mod projection_semantics;
pub mod cultural_projection_audit_v3;
pub mod cultural_projection_audit_v4;
pub mod dkg;
pub mod temporal_projection;
pub mod constants;
pub mod data;
pub mod economics;
pub mod energy_trading;
pub mod geo;
pub mod lod;
pub mod mycelix_flows;
// pub mod relativity_viz;
pub mod geometry;
pub mod math;
pub mod picking;
pub mod simulation;
pub mod solar_system;
pub mod timeline;
pub mod types;
pub mod visual_validation;

pub use types::*;
