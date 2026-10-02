// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).

pub mod aesthetics;
pub mod civilizational;
pub mod confluence;
pub mod constants;
pub mod cultural_argumentation;
pub mod cultural_projection_audit_v3;
pub mod cultural_projection_audit_v4;
pub mod cultural_projection_audit_v5;
pub mod cultural_projection_historical_receipt;
pub mod cultural_projection_historical_replay;
pub mod cultural_projection_replay;
pub mod cultural_systems;
pub mod data;
pub mod dkg;
pub mod economics;
pub mod energy_trading;
pub mod geo;
pub mod lod;
pub mod mycelix_flows;
pub mod ontology_context;
pub mod ontology_mapping;
pub mod ontology_resolution;
pub mod projection_semantics;
pub mod temporal_projection;
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
