// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Shared data types, geodetic math, geometry generation, and constants
//! for Sol Atlas renderers (Leptos WebGL, Bevy wgpu).
//!
//! All math uses raw `[f32; N]` arrays to avoid dependency collisions
//! between rendering backends (web-sys, glam/nalgebra).

pub mod aesthetics;
pub mod built_environment_case;
/// Versioned source-owner descriptor catalogues, cross-case identity
/// reconciliation, and loss-accounted metadata validation. The catalogue is
/// case-bound, but its owner-native identity hashes deliberately exclude case
/// membership so shared source records retain stable identity across projects.
/// Structural validation does not authenticate bytes, owners, rights, signatures,
/// coverage completeness, engineering correctness, or authorization; those remain
/// independent evidence and trust obligations.
pub mod built_environment_source_descriptor;
pub mod confluence;
pub mod constants;
pub mod data;
pub mod economics;
pub mod energy_trading;
pub mod geo;
pub mod industry_assurance_projection;
pub mod lod;
pub mod mycelix_flows;
/// Bounded streaming rehashing of source payload bytes referenced by
/// owner-native descriptors. Byte equality is not origin/trust/rights proof.
/// Bounded streaming rehash of descriptor-referenced bytes. This verifies
/// digest equality against the declared payload, not issuer, owner, rights,
/// source-snapshot completeness, engineering correctness, or action authority.
pub mod source_payload_verifier;
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
