// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::investigation_campaign::{
    score_investigation_campaign, CampaignPolicyMetrics, InvestigationCampaignReceipt,
    InvestigationTournamentCampaign,
};
use std::env;
use std::fs;

const FIXTURE: &[u8] = include_bytes!(
    "../../research/earth-observability/fixtures/investigation-campaign-synthetic-v0.1.json"
);
const EXPECTED_METRICS: &str = include_str!(
    "../../research/earth-observability/baselines/investigation-campaign-baseline-v0.1.json"
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvestigationCampaignReceiptWithDigest {
    benchmark_id: String,
    input_sha256: String,
    receipt: InvestigationCampaignReceipt,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(input: &[u8]) -> Result<InvestigationCampaignReceiptWithDigest, String> {
    let campaign: InvestigationTournamentCampaign =
        serde_json::from_slice(input).map_err(|error| format!("campaign JSON: {error}"))?;
    let receipt = score_investigation_campaign(&campaign)
        .map_err(|errors| format!("campaign invalid: {errors:?}"))?;
    Ok(InvestigationCampaignReceiptWithDigest {
        benchmark_id: "EARTH-OBSERVABILITY-INVESTIGATION-CAMPAIGN-001".to_string(),
        input_sha256: sha256_hex(input),
        receipt,
    })
}

fn check_pinned_fixture() -> Result<(), String> {
    let receipt = build_receipt(FIXTURE)?;
    let expected: Vec<CampaignPolicyMetrics> = serde_json::from_str(EXPECTED_METRICS)
        .map_err(|error| format!("pinned campaign metrics JSON: {error}"))?;
    if receipt.receipt.policy_metrics != expected {
        return Err(format!(
            "pinned campaign metrics differ\nexpected: {expected:#?}\nobserved: {:#?}",
            receipt.receipt.policy_metrics
        ));
    }
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("--check-pinned") => {
            if args.next().is_some() {
                eprintln!("unexpected extra arguments");
                std::process::exit(2);
            }
            match check_pinned_fixture() {
                Ok(()) => println!("PINNED_INVESTIGATION_CAMPAIGN_BASELINE_PASS"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        Some(path) => {
            if args.next().is_some() {
                eprintln!("unexpected extra arguments");
                std::process::exit(2);
            }
            let input = match fs::read(path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    eprintln!("cannot read {path}: {error}");
                    std::process::exit(1);
                }
            };
            match build_receipt(&input) {
                Ok(receipt) => println!(
                    "{}",
                    serde_json::to_string_pretty(&receipt).expect("receipt serialization")
                ),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("usage: investigation_campaign_eval <campaign.json> | --check-pinned");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_receipt_preserves_project_level_outcomes() {
        let receipt = build_receipt(FIXTURE).expect("campaign fixture scores");
        assert_eq!(receipt.benchmark_id, "EARTH-OBSERVABILITY-INVESTIGATION-CAMPAIGN-001");
        assert_eq!(receipt.input_sha256, sha256_hex(FIXTURE));
        assert_eq!(receipt.receipt.project_count, 3);
        assert_eq!(receipt.receipt.project_results.len(), 15);
    }

    #[test]
    fn pinned_campaign_baseline_is_reproducible() {
        check_pinned_fixture().expect("campaign baseline matches");
    }

    #[test]
    fn input_byte_change_changes_campaign_digest() {
        let original = build_receipt(FIXTURE).expect("fixture scores");
        let mut changed = FIXTURE.to_vec();
        changed.push(b' ');
        let changed_receipt = build_receipt(&changed).expect("trailing whitespace parses");
        assert_ne!(original.input_sha256, changed_receipt.input_sha256);
    }
}
