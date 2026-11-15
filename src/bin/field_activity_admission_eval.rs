// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::field_activity_admission::{
    ActivityTrackAdmissibility, FieldActivityAdmission, assess_field_activity,
};
use std::env;
use std::fs;

const FIXTURE: &[u8] = include_bytes!(
    "../../research/earth-observability/fixtures/field-activity-admissibility-synthetic-v0.1.json"
);
const EXPECTED: &str = include_str!(
    "../../research/earth-observability/baselines/field-activity-admissibility-synthetic-v0.1.json"
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldActivityAdmissionReceipt {
    benchmark_id: String,
    evaluator_version: String,
    input_sha256: String,
    assessments: Vec<ActivityTrackAdmissibility>,
    status: String,
    claim_boundary: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_receipt(input: &[u8]) -> Result<FieldActivityAdmissionReceipt, String> {
    let activities: Vec<FieldActivityAdmission> = serde_json::from_slice(input)
        .map_err(|error| format!("field activity admission JSON: {error}"))?;
    let mut assessments = Vec::with_capacity(activities.len());
    for activity in &activities {
        let assessment = assess_field_activity(activity).map_err(|errors| {
            format!(
                "activity {} is structurally invalid: {errors:?}",
                activity.activity_id
            )
        })?;
        assessments.push(assessment);
    }

    Ok(FieldActivityAdmissionReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-FIELD-ACTIVITY-ADMISSION-001".to_string(),
        evaluator_version: "0.1.0".to_string(),
        input_sha256: sha256_hex(input),
        assessments,
        status: "synthetic_per_activity_track_admissibility_not_performance".to_string(),
        claim_boundary: "This receipt classifies metric eligibility from declared metadata and reference truth tiers. It does not establish detection performance, field POD, geometric accuracy, complete network truth, physical sensor qualification or engineering acceptance.".to_string(),
    })
}

fn check_pinned() -> Result<(), String> {
    let receipt = build_receipt(FIXTURE)?;
    let expected: Vec<ActivityTrackAdmissibility> = serde_json::from_str(EXPECTED)
        .map_err(|error| format!("pinned eligibility JSON: {error}"))?;
    if receipt.assessments != expected {
        return Err(format!(
            "pinned activity eligibility differs\nexpected: {expected:#?}\nobserved: {:#?}",
            receipt.assessments
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
                println!("PINNED_FIELD_ACTIVITY_ADMISSION_PASS");
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
            eprintln!("usage: field_activity_admission_eval <activities.json> | --check-pinned");
            std::process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!("unexpected extra arguments");
        std::process::exit(2);
    }

    let input = match fs::read(&input_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {input_path}: {error}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_fixture_emits_six_per_activity_track_assessments() {
        let receipt = build_receipt(FIXTURE).expect("fixture produces admission report");
        assert_eq!(receipt.assessments.len(), 6);
        assert_eq!(receipt.input_sha256, sha256_hex(FIXTURE));
        assert_eq!(
            receipt.status,
            "synthetic_per_activity_track_admissibility_not_performance"
        );
    }

    #[test]
    fn positionless_and_wheel_compromised_activities_remain_event_eligible_only() {
        let receipt = build_receipt(FIXTURE).unwrap();
        let wheel = receipt
            .assessments
            .iter()
            .find(|item| item.activity_id == "activity-wheel-compromised")
            .unwrap();
        let positionless = receipt
            .assessments
            .iter()
            .find(|item| item.activity_id == "activity-positionless-radargram")
            .unwrap();
        assert_eq!(
            wheel.signal_event_detection,
            positionless.signal_event_detection
        );
        assert!(matches!(
            wheel.localization,
            sol_atlas_core::field_activity_admission::TrackEligibility::Ineligible(_)
        ));
        assert!(matches!(
            positionless.localization,
            sol_atlas_core::field_activity_admission::TrackEligibility::Ineligible(_)
        ));
    }

    #[test]
    fn owner_record_only_truth_is_excluded_from_physical_scoring() {
        let receipt = build_receipt(FIXTURE).unwrap();
        let owner_record = receipt
            .assessments
            .iter()
            .find(|item| item.activity_id == "activity-owner-record-only")
            .unwrap();
        assert!(matches!(
            owner_record.signal_event_detection,
            sol_atlas_core::field_activity_admission::TrackEligibility::Ineligible(_)
        ));
    }

    #[test]
    fn inspected_negative_is_scoped_and_admitted_only_for_supported_tracks() {
        let receipt = build_receipt(FIXTURE).unwrap();
        let negative = receipt
            .assessments
            .iter()
            .find(|item| item.activity_id == "activity-inspected-negative-support")
            .unwrap();
        assert_eq!(
            negative.signal_event_detection,
            sol_atlas_core::field_activity_admission::TrackEligibility::Eligible
        );
        assert_eq!(
            negative.localization,
            sol_atlas_core::field_activity_admission::TrackEligibility::Eligible
        );
        assert!(matches!(
            negative.network_topology_3d,
            sol_atlas_core::field_activity_admission::TrackEligibility::Ineligible(_)
        ));
    }

    #[test]
    fn pinned_activity_admissibility_baseline_is_reproducible() {
        check_pinned().expect("pinned activity eligibility matches");
    }
}
