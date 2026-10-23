// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde_json::json;
use sol_atlas_core::energy_design_adapter::{
    DispatchMappingConfig, compile_energy_candidate,
};
use std::{env, error::Error, fs, io};

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: energy_design_dispatch_compile <candidate.json> <mapping-config.json>",
        )
        .into());
    }

    let candidate_bytes = fs::read(&args[1])?;
    let config_bytes = fs::read(&args[2])?;
    let config: DispatchMappingConfig = serde_json::from_slice(&config_bytes)?;
    let compilation = compile_energy_candidate(&candidate_bytes, &config)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    println!("{}", serde_json::to_string_pretty(&compilation)?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("energy design dispatch compilation failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const CANDIDATE: &str =
        include_str!("../../../schemas/examples/energy-design-pv-battery-grid.synthetic.json");
    const CONFIG: &str =
        include_str!("../../../schemas/examples/energy-dispatch-mapping-config.synthetic.json");

    fn sha256_hex(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn fixture_compilation_emits_reproducible_digests_and_pending_evaluations() {
        let candidate = serde_json::from_str::<serde_json::Value>(CANDIDATE).unwrap();
        let config = serde_json::from_str::<DispatchMappingConfig>(CONFIG).unwrap();
        let compilation = compile_energy_candidate(CANDIDATE.as_bytes(), &config).unwrap();

        assert_eq!(
            compilation.candidate_id,
            candidate["candidate_id"].as_str().unwrap()
        );
        assert_eq!(
            compilation.candidate_sha256,
            sha256_hex(CANDIDATE.as_bytes())
        );
        assert_eq!(compilation.dispatch_request.schema_version, "energy-dispatch-v0.1");
        assert_eq!(compilation.dispatch_request_sha256, sha256_hex(
            &serde_json::to_vec(&compilation.dispatch_request).unwrap()
        ));
        assert_eq!(
            compilation.requested_evaluations_not_executed,
            candidate["requested_evaluations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        );
        assert!(compilation.objective_coverage.iter().any(|item| {
            item.objective_id == "operating_cost" && item.status == "partial"
        }));
        assert!(compilation.objective_coverage.iter().any(|item| {
            item.objective_id == "essential_unserved_energy" && item.status == "mapped"
        }));

        let recompiled = compile_energy_candidate(CANDIDATE.as_bytes(), &config).unwrap();
        assert_eq!(compilation, recompiled);
        let output = serde_json::to_value(compilation).unwrap();
        assert_eq!(output["synthetic"], json!(true));
        assert!(output["claim_ceiling"].as_str().unwrap().contains("does not prove"));
    }

    #[test]
    fn malformed_mapping_config_fails_closed() {
        let config: DispatchMappingConfig = serde_json::from_str(CONFIG).unwrap();
        let candidate = serde_json::from_str::<serde_json::Value>(CANDIDATE).unwrap();
        let mut invalid = config;
        invalid.factor_basis.clear();
        let raw = serde_json::to_vec(&candidate).unwrap();
        assert!(compile_energy_candidate(&raw, &invalid).is_err());
    }
}
