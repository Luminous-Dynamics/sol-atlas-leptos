// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::evidence_revalidation::{
    EvidenceDependencyGraph, RevalidationPlan, build_revalidation_plan,
};
use std::env;
use std::fs;

const FIXTURE: &[u8] = include_bytes!(
    "../../research/earth-observability/fixtures/evidence-dependency-graph-synthetic-v0.1.json"
);
const EXPECTED: &str = include_str!(
    "../../research/earth-observability/baselines/revalidation-plan-source-change-synthetic-v0.1.json"
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevalidationReceipt {
    benchmark_id: String,
    evaluator_version: String,
    input_sha256: String,
    plan: RevalidationPlan,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(input: &[u8], changed_node_ids: &[String]) -> Result<RevalidationReceipt, String> {
    let graph: EvidenceDependencyGraph =
        serde_json::from_slice(input).map_err(|error| format!("dependency graph JSON: {error}"))?;
    let plan = build_revalidation_plan(&graph, changed_node_ids)
        .map_err(|errors| format!("dependency graph or change set invalid: {errors:?}"))?;

    Ok(RevalidationReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-REVALIDATION-001".to_string(),
        evaluator_version: "0.1.0".to_string(),
        input_sha256: sha256_hex(input),
        plan,
    })
}

fn check_pinned() -> Result<(), String> {
    let receipt = build_receipt(FIXTURE, &["source:gpr-survey-a".to_string()])?;
    let expected: RevalidationPlan =
        serde_json::from_str(EXPECTED).map_err(|error| format!("pinned plan JSON: {error}"))?;
    if receipt.plan != expected {
        return Err(format!(
            "pinned revalidation plan differs\nexpected: {expected:#?}\nobserved: {:#?}",
            receipt.plan
        ));
    }
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let first = args.next();
    if first.as_deref() == Some("--check-pinned") {
        if args.next().is_some() {
            eprintln!("--check-pinned accepts no extra arguments");
            std::process::exit(2);
        }
        match check_pinned() {
            Ok(()) => {
                println!("PINNED_REVALIDATION_PLAN_PASS");
                return;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }

    let input_path = match first {
        Some(value) => value,
        None => {
            eprintln!(
                "usage: evidence_revalidation_eval <graph.json> <changed_node_id> [more_changed_ids...] | --check-pinned"
            );
            std::process::exit(2);
        }
    };
    let changed_node_ids: Vec<String> = args.collect();
    if changed_node_ids.is_empty() {
        eprintln!("at least one changed_node_id is required");
        std::process::exit(2);
    }

    let input = match fs::read(&input_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {input_path}: {error}");
            std::process::exit(1);
        }
    };
    match build_receipt(&input, &changed_node_ids) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_change_emits_pinned_downstream_invalidation_receipt() {
        let receipt = build_receipt(FIXTURE, &["source:gpr-survey-a".to_string()])
            .expect("source change produces plan");
        assert_eq!(receipt.benchmark_id, "EARTH-OBSERVABILITY-REVALIDATION-001");
        assert_eq!(receipt.input_sha256, sha256_hex(FIXTURE));
        assert_eq!(receipt.plan.invalidated_node_ids.len(), 7);
        assert_eq!(
            receipt.plan.revalidation_order.first().unwrap(),
            "transform:route-a"
        );
    }

    #[test]
    fn pinned_revalidation_closure_is_reproducible() {
        check_pinned().expect("pinned dependency closure matches");
    }

    #[test]
    fn receipt_binds_exact_graph_bytes() {
        let original = build_receipt(FIXTURE, &["source:gpr-survey-a".to_string()]).unwrap();
        let mut changed = FIXTURE.to_vec();
        changed.push(b' ');
        let revised = build_receipt(&changed, &["source:gpr-survey-a".to_string()]).unwrap();
        assert_ne!(original.input_sha256, revised.input_sha256);
    }

    #[test]
    fn unknown_changed_node_is_rejected() {
        assert!(build_receipt(FIXTURE, &["source:missing".to_string()]).is_err());
    }
}
