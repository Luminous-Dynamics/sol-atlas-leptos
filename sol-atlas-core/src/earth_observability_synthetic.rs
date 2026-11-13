// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Deterministic, abstract subsurface profile fixtures for contract and
//! algorithm-development tests. These values are not simulated instrument
//! physics and must never be represented as field measurements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticProfileDomain {
    pub x_min_m: f64,
    pub x_max_m: f64,
    pub depth_min_m: f64,
    pub depth_max_m: f64,
    pub x_step_m: f64,
    pub depth_reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntheticSplitRole {
    Fit,
    IndependentValidation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntheticObservationState {
    Present,
    Unobserved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticBoreholeObservation {
    pub observation_id: String,
    pub borehole_id: String,
    pub site_id: String,
    pub x_m: f64,
    pub interface_depth_m: f64,
    pub standard_uncertainty_m: f64,
    pub split_role: SyntheticSplitRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticGeophysicalObservation {
    pub observation_id: String,
    pub source_id: String,
    pub site_id: String,
    pub modality: String,
    pub x_m: f64,
    pub interface_depth_estimate_m: Option<f64>,
    pub standard_uncertainty_m: Option<f64>,
    pub state: SyntheticObservationState,
    pub split_role: SyntheticSplitRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticValidationTruth {
    pub validation_id: String,
    pub site_id: String,
    pub x_m: f64,
    pub true_interface_depth_m: f64,
    pub split_role: SyntheticSplitRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticProfileFixture {
    pub fixture_id: String,
    pub fixture_version: String,
    pub seed: u64,
    pub status: String,
    pub geography: String,
    pub claim_boundary: String,
    pub domain: SyntheticProfileDomain,
    pub borehole_observations: Vec<SyntheticBoreholeObservation>,
    pub geophysical_observations: Vec<SyntheticGeophysicalObservation>,
    pub independent_validation_truth: Vec<SyntheticValidationTruth>,
}

const X_POSITIONS_M: [u32; 11] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
const INTERFACE_WAVE_CM: [i32; 11] = [0, 18, 31, 16, -12, -26, -9, 22, 34, 8, -16];

fn interface_depth_cm(site_offset_cm: i32, x_index: usize) -> i32 {
    850 + (X_POSITIONS_M[x_index] as i32 * 4) + INTERFACE_WAVE_CM[x_index] + site_offset_cm
}

fn next_noise_cm(state: &mut u64) -> i32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (((*state >> 32) % 41) as i32) - 20
}

/// Create a deterministic two-transect layered-interface challenge.
///
/// The training transect has sparse borehole contacts and noisy geophysical
/// proxy observations. The blind transect has a shifted interface, partial
/// geophysical coverage, and separate independent validation truth. The
/// "surface_wave_proxy" values are abstract noisy interface estimates, not
/// synthetic waveforms or a physical surface-wave forward model.
pub fn generate_synthetic_profile_fixture(seed: u64) -> SyntheticProfileFixture {
    let mut random_state = seed;
    let mut borehole_observations = Vec::new();
    let mut geophysical_observations = Vec::new();
    let mut independent_validation_truth = Vec::new();

    for (site_id, site_offset_cm, role) in [
        ("synthetic-training-transect", 0, SyntheticSplitRole::Fit),
        ("synthetic-blind-transect", 125, SyntheticSplitRole::IndependentValidation),
    ] {
        for (index, x_m) in X_POSITIONS_M.iter().enumerate() {
            let true_depth_cm = interface_depth_cm(site_offset_cm, index);

            if site_id == "synthetic-training-transect" && index % 2 == 0 {
                borehole_observations.push(SyntheticBoreholeObservation {
                    observation_id: format!("bh-observation-{x_m:03}"),
                    borehole_id: format!("bh-{x_m:03}"),
                    site_id: site_id.to_string(),
                    x_m: *x_m as f64,
                    interface_depth_m: true_depth_cm as f64 / 100.0,
                    standard_uncertainty_m: 0.03,
                    split_role: SyntheticSplitRole::Fit,
                });
            }

            let is_blind_zone = site_id == "synthetic-blind-transect" && *x_m >= 60;
            let (estimate, uncertainty, state) = if is_blind_zone {
                (None, None, SyntheticObservationState::Unobserved)
            } else {
                let noise_cm = next_noise_cm(&mut random_state);
                (
                    Some((true_depth_cm + noise_cm) as f64 / 100.0),
                    Some(if site_id == "synthetic-training-transect" {
                        0.45
                    } else {
                        0.65
                    }),
                    SyntheticObservationState::Present,
                )
            };

            geophysical_observations.push(SyntheticGeophysicalObservation {
                observation_id: format!(
                    "sw-proxy-{}-{x_m:03}",
                    if site_id == "synthetic-training-transect" {
                        "train"
                    } else {
                        "blind"
                    }
                ),
                source_id: format!(
                    "synthetic-surface-wave-proxy-{}",
                    if site_id == "synthetic-training-transect" {
                        "train"
                    } else {
                        "blind"
                    }
                ),
                site_id: site_id.to_string(),
                modality: "surface_wave_interface_proxy".to_string(),
                x_m: *x_m as f64,
                interface_depth_estimate_m: estimate,
                standard_uncertainty_m: uncertainty,
                state,
                split_role: role.clone(),
            });

            if site_id == "synthetic-blind-transect" {
                independent_validation_truth.push(SyntheticValidationTruth {
                    validation_id: format!("blind-truth-{x_m:03}"),
                    site_id: site_id.to_string(),
                    x_m: *x_m as f64,
                    true_interface_depth_m: true_depth_cm as f64 / 100.0,
                    split_role: SyntheticSplitRole::IndependentValidation,
                });
            }
        }
    }

    SyntheticProfileFixture {
        fixture_id: "EARTH-OBSERVABILITY-PROFILE-SYNTHETIC-001".to_string(),
        fixture_version: "0.1.0".to_string(),
        seed,
        status: "synthetic_generated_not_field_data".to_string(),
        geography: "abstract_non_geographic".to_string(),
        claim_boundary: "Synthetic layered interface and noisy proxy observations only; no field location, sensor physics, engineering properties, or safety determination.".to_string(),
        domain: SyntheticProfileDomain {
            x_min_m: 0.0,
            x_max_m: 100.0,
            depth_min_m: 0.0,
            depth_max_m: 30.0,
            x_step_m: 10.0,
            depth_reference: "synthetic_local_depth_datum".to_string(),
        },
        borehole_observations,
        geophysical_observations,
        independent_validation_truth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMITTED_FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json"
    );

    #[test]
    fn generator_is_deterministic_for_a_frozen_seed() {
        let a = generate_synthetic_profile_fixture(20_261_113);
        let b = generate_synthetic_profile_fixture(20_261_113);
        assert_eq!(a, b);
    }

    #[test]
    fn seed_changes_the_generated_observation_noise() {
        let a = generate_synthetic_profile_fixture(20_261_113);
        let b = generate_synthetic_profile_fixture(20_261_114);
        assert_ne!(
            a.geophysical_observations,
            b.geophysical_observations
        );
    }

    #[test]
    fn blind_truth_is_separate_from_training_boreholes() {
        let fixture = generate_synthetic_profile_fixture(20_261_113);
        assert_eq!(fixture.borehole_observations.len(), 6);
        assert_eq!(fixture.independent_validation_truth.len(), 11);
        assert!(fixture
            .borehole_observations
            .iter()
            .all(|sample| sample.split_role == SyntheticSplitRole::Fit));
        assert!(fixture
            .independent_validation_truth
            .iter()
            .all(|sample| sample.split_role == SyntheticSplitRole::IndependentValidation));
        assert!(fixture.independent_validation_truth.iter().all(|truth| !fixture
            .borehole_observations
            .iter()
            .any(|sample| sample.site_id == truth.site_id)));
    }

    #[test]
    fn blind_zone_remains_unobserved_and_is_not_zero_filled() {
        let fixture = generate_synthetic_profile_fixture(20_261_113);
        let blind_zone: Vec<&SyntheticGeophysicalObservation> = fixture
            .geophysical_observations
            .iter()
            .filter(|sample| {
                sample.site_id == "synthetic-blind-transect" && sample.x_m >= 60.0
            })
            .collect();
        assert_eq!(blind_zone.len(), 5);
        assert!(blind_zone.iter().all(|sample| {
            sample.state == SyntheticObservationState::Unobserved
                && sample.interface_depth_estimate_m.is_none()
                && sample.standard_uncertainty_m.is_none()
        }));
    }

    #[test]
    fn fixture_is_explicitly_non_geographic_and_non_physical() {
        let fixture = generate_synthetic_profile_fixture(20_261_113);
        assert_eq!(fixture.geography, "abstract_non_geographic");
        assert_eq!(fixture.status, "synthetic_generated_not_field_data");
        assert!(fixture.claim_boundary.contains("no field location"));
    }

    #[test]
    fn geophysical_proxy_noise_respects_declared_uncertainty_bound() {
        let fixture = generate_synthetic_profile_fixture(20_261_113);
        for observation in fixture.geophysical_observations.iter().filter(|sample| {
            sample.state == SyntheticObservationState::Present
        }) {
            let truth = fixture
                .independent_validation_truth
                .iter()
                .find(|sample| {
                    sample.site_id == observation.site_id && sample.x_m == observation.x_m
                });
            let training_truth_cm = interface_depth_cm(0, X_POSITIONS_M.iter().position(|x| *x as f64 == observation.x_m).unwrap());
            let truth_depth = if observation.site_id == "synthetic-training-transect" {
                training_truth_cm as f64 / 100.0
            } else {
                truth.expect("blind validation truth must exist").true_interface_depth_m
            };
            let error = (observation.interface_depth_estimate_m.unwrap() - truth_depth).abs();
            assert!(error <= 0.20, "proxy noise exceeded 0.20 m");
            assert!(observation.standard_uncertainty_m.is_some());
        }
    }

    #[test]
    fn committed_fixture_matches_the_seeded_generator() {
        let fixture: SyntheticProfileFixture =
            serde_json::from_str(COMMITTED_FIXTURE).expect("fixture must parse");
        assert_eq!(fixture, generate_synthetic_profile_fixture(20_261_113));
    }
}
