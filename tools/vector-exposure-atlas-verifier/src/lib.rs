use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA: &str = "luminous.vector_exposure_atlas_cases.v0.1";
pub const STATUS: &str = "synthetic_only_no_personal_health_data";
pub const EXPECTED_FIXTURE_SHA256: &str = "a8a51568870356475e057fe613fb5e15295d1cf1246e2d8eeefdf54d10b3e751";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub status: String,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub name: String,
    pub expected_state: String,
    pub assertions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub report_schema: &'static str,
    pub status: &'static str,
    pub source_sha256: String,
    pub case_count: usize,
    pub errors: Vec<String>,
    pub claim_ceiling: &'static str,
}

const REQUIRED: [(&str, &str, &[&str]); 11] = [
    ("VA-001", "household_projection_rejected", &["source_spatial_scale_preserved", "no_false_precision"]),
    ("VA-002", "stale_or_unknown", &["observation_time_preserved", "stale_not_zero"]),
    ("VA-003", "environmental_covariate_only", &["no_vector_presence_claim"]),
    ("VA-004", "vector_presence_only", &["no_infection_or_disease_claim"]),
    ("VA-005", "coverage_bias_disclosed", &["effort_denominator_preserved", "no_raw_count_ranking"]),
    ("VA-006", "rejected_or_quarantined", &["crs_validated", "transformation_provenance_retained"]),
    ("VA-007", "species_conflict_preserved", &["taxonomic_confidence_preserved", "no_species_coalescence"]),
    ("VA-008", "generalized_or_omitted_from_public_view", &["sensitivity_policy_applied", "small_cell_suppression"]),
    ("VA-009", "quarantined", &["licence_checked_before_ingest", "no_public_redistribution"]),
    ("VA-010", "causal_effect_not_admitted", &["sampling_protocol_difference_exposed", "no_causal_claim"]),
    ("VA-011", "rejected", &["population_completeness_required", "missingness_visible"]),
];

pub fn verify(bytes: &[u8]) -> Report {
    let source_sha256 = format!("{:x}", Sha256::digest(bytes));
    let mut errors = Vec::new();
    if source_sha256 != EXPECTED_FIXTURE_SHA256 {
        errors.push(format!(
            "fixture_digest_mismatch: expected {}, got {}",
            EXPECTED_FIXTURE_SHA256,
            source_sha256
        ));
    }
    let mut case_count = 0;

    match serde_json::from_slice::<Manifest>(bytes) {
        Err(error) => errors.push(format!("strict_json_decode: {error}")),
        Ok(manifest) => {
            case_count = manifest.cases.len();
            if manifest.schema != SCHEMA { errors.push("schema_mismatch".into()); }
            if manifest.status != STATUS { errors.push("status_mismatch".into()); }
            if manifest.cases.len() != REQUIRED.len() {
                errors.push(format!("case_count_mismatch: expected {}, got {}", REQUIRED.len(), manifest.cases.len()));
            }
            let expected: BTreeMap<&str, (&str, &[&str])> = REQUIRED.iter()
                .map(|(id, state, assertions)| (*id, (*state, *assertions))).collect();
            let mut seen = BTreeSet::new();

            for case in &manifest.cases {
                if case.id.trim().is_empty() || !seen.insert(case.id.as_str()) {
                    errors.push(format!("empty_or_duplicate_id: {}", case.id));
                }
                if case.name.trim().is_empty() || case.assertions.is_empty() {
                    errors.push(format!("incomplete_case: {}", case.id));
                }
                let actual: BTreeSet<&str> = case.assertions.iter().map(String::as_str).collect();
                if actual.len() != case.assertions.len() {
                    errors.push(format!("duplicate_assertion: {}", case.id));
                }
                match expected.get(case.id.as_str()) {
                    None => errors.push(format!("unexpected_case: {}", case.id)),
                    Some((state, assertions)) => {
                        if case.expected_state != *state {
                            errors.push(format!("state_mismatch: {} expected {}, got {}", case.id, state, case.expected_state));
                        }
                        for assertion in *assertions {
                            if !actual.contains(assertion) {
                                errors.push(format!("required_assertion_missing: {} -> {}", case.id, assertion));
                            }
                        }
                    }
                }
            }
            for (id, _, _) in REQUIRED {
                if !seen.contains(id) { errors.push(format!("missing_case: {id}")); }
            }
        }
    }

    errors.sort();
    Report {
        report_schema: "luminous.vector_exposure_atlas_verification.v0.1",
        status: if errors.is_empty() { "CONFORMS" } else { "REJECTED" },
        source_sha256,
        case_count,
        errors,
        claim_ceiling: "synthetic_fixture_semantic_conformance_only",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../../../docs/research/vector-exposure-atlas/VECTOR_EXPOSURE_CASES_V0_1.json");

    #[test]
    fn frozen_fixture_conforms() {
        let report = verify(FIXTURE.as_bytes());
        assert_eq!(report.status, "CONFORMS", "{:?}", report.errors);
        assert_eq!(report.case_count, REQUIRED.len());
        assert_eq!(report.source_sha256, EXPECTED_FIXTURE_SHA256);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let changed = FIXTURE.replace("\"id\": \"VA-001\"", "\"id\": \"VA-001\", \"extra\": true");
        assert_ne!(changed, FIXTURE);
        let report = verify(changed.as_bytes());
        assert_eq!(report.status, "REJECTED");
        assert!(report.errors.iter().any(|e| e.starts_with("strict_json_decode:")));
    }

    #[test]
    fn environmental_suitability_is_not_vector_presence() {
        let changed = FIXTURE.replace(
            "\"expected_state\": \"environmental_covariate_only\"",
            "\"expected_state\": \"confirmed_vector_presence\"",
        );
        assert_ne!(changed, FIXTURE);
        let report = verify(changed.as_bytes());
        assert_eq!(report.status, "REJECTED");
        assert!(report.errors.iter().any(|e| e.starts_with("state_mismatch: VA-003")));
    }

    #[test]
    fn vector_presence_is_not_disease_incidence() {
        let changed = FIXTURE.replace(
            "\"expected_state\": \"vector_presence_only\"",
            "\"expected_state\": \"disease_incidence_confirmed\"",
        );
        assert_ne!(changed, FIXTURE);
        let report = verify(changed.as_bytes());
        assert_eq!(report.status, "REJECTED");
        assert!(report.errors.iter().any(|e| e.starts_with("state_mismatch: VA-004")));
    }

    #[test]
    fn protected_household_data_cannot_be_public() {
        let changed = FIXTURE.replace(
            "\"expected_state\": \"generalized_or_omitted_from_public_view\"",
            "\"expected_state\": \"public_household_coordinates\"",
        );
        assert_ne!(changed, FIXTURE);
        let report = verify(changed.as_bytes());
        assert_eq!(report.status, "REJECTED");
        assert!(report.errors.iter().any(|e| e.starts_with("state_mismatch: VA-008")));
    }
    #[test]
    fn fixture_byte_change_is_rejected_by_digest() {
        let changed = FIXTURE.replace("ward_resolution_not_household_resolution", "ward_resolution_not_household_resolution ");
        assert_ne!(changed, FIXTURE);
        let report = verify(changed.as_bytes());
        assert_eq!(report.status, "REJECTED");
        assert!(report.errors.iter().any(|e| e.starts_with("fixture_digest_mismatch:")));
    }

}
