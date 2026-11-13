// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generate or verify the committed synthetic Earth Observability profile.
//!
//! This program is an offline fixture tool. It does not generate physical
//! sensor data, transform geospatial coordinates, or estimate geological truth.

use sol_atlas_core::earth_observability_synthetic::{
    SyntheticProfileFixture, generate_synthetic_profile_fixture,
};
use std::env;
use std::fs;
use std::path::Path;

const FIXTURE_PATH: &str =
    "research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json";
const FIXTURE_SEED: u64 = 20_261_113;

fn usage(program: &str) {
    eprintln!("Usage: {program} [--check] [FIXTURE.json]");
    eprintln!("Default fixture: {FIXTURE_PATH}");
}

fn main() {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "earth_observability_generate".to_string());

    let first = args.next();
    let (check, path) = match first.as_deref() {
        None => (false, FIXTURE_PATH.to_string()),
        Some("--help" | "-h") => {
            usage(&program);
            return;
        }
        Some("--check") => {
            let path = args.next().unwrap_or_else(|| FIXTURE_PATH.to_string());
            (true, path)
        }
        Some(path) => (false, path.to_string()),
    };

    if args.next().is_some() {
        usage(&program);
        std::process::exit(64);
    }

    let expected = generate_synthetic_profile_fixture(FIXTURE_SEED);
    if check {
        let result = fs::read(Path::new(&path))
            .map_err(|error| format!("cannot read {path}: {error}"))
            .and_then(|bytes| {
                serde_json::from_slice::<SyntheticProfileFixture>(&bytes)
                    .map_err(|error| format!("fixture parse failed: {error}"))
            });

        match result {
            Ok(actual) if actual == expected => {
                println!(
                    "PASS: {path} matches seed {FIXTURE_SEED}; {} boreholes, {} geophysical proxies, {} independent truth points",
                    actual.borehole_observations.len(),
                    actual.geophysical_observations.len(),
                    actual.independent_validation_truth.len()
                );
            }
            Ok(_) => {
                eprintln!("FAIL: {path} differs from the deterministic generator output");
                std::process::exit(2);
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    } else {
        let mut bytes =
            serde_json::to_vec_pretty(&expected).expect("synthetic fixture must serialize");
        bytes.push(b'\n');

        if let Some(parent) = Path::new(&path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(error) = fs::create_dir_all(parent) {
                    eprintln!("cannot create fixture directory: {error}");
                    std::process::exit(1);
                }
            }
        }

        if let Err(error) = fs::write(Path::new(&path), bytes) {
            eprintln!("cannot write {path}: {error}");
            std::process::exit(1);
        }

        println!("WROTE: {path} from deterministic seed {FIXTURE_SEED}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_profile_fixture_round_trips() {
        let fixture = generate_synthetic_profile_fixture(FIXTURE_SEED);
        let bytes = serde_json::to_vec(&fixture).expect("fixture serializes");
        let parsed: SyntheticProfileFixture =
            serde_json::from_slice(&bytes).expect("fixture parses");
        assert_eq!(parsed, fixture);
        assert_eq!(parsed.borehole_observations.len(), 6);
        assert_eq!(parsed.geophysical_observations.len(), 22);
        assert_eq!(parsed.independent_validation_truth.len(), 11);
    }

    #[test]
    fn unobserved_fixture_points_remain_null() {
        let fixture = generate_synthetic_profile_fixture(FIXTURE_SEED);
        let unobserved: Vec<_> = fixture
            .geophysical_observations
            .iter()
            .filter(|sample| sample.interface_depth_estimate_m.is_none())
            .collect();
        assert_eq!(unobserved.len(), 5);
        assert!(
            unobserved
                .iter()
                .all(|sample| sample.standard_uncertainty_m.is_none())
        );
    }
}
