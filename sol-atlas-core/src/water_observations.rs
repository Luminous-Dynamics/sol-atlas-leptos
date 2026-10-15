// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Semantic checks for source-attributed public water observations.
//!
//! Shape validation belongs to water-public-observation-bundle-v0.1.schema.json.
//! This module additionally closes source references, enforces unique identifiers,
//! and prevents reversed observation windows. It does not authenticate publishers
//! or independently verify the source's reported measurements.

use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationIssue {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObservationValidationReport {
    pub issues: Vec<ObservationIssue>,
}

impl ObservationValidationReport {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    fn push(&mut self, code: &'static str, path: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ObservationIssue {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

/// Checks semantic consistency of the public observation bundle.
/// The Draft 2020-12 schema must also be applied as a separate gate.
pub fn validate_public_observation_bundle(bundle: &Value) -> ObservationValidationReport {
    let mut report = ObservationValidationReport::default();
    let Some(object) = bundle.as_object() else {
        report.push("BUNDLE_NOT_OBJECT", "$", "bundle must be a JSON object");
        return report;
    };

    if object.get("schema_version").and_then(Value::as_str)
        != Some("water-public-observation-bundle/v0.1")
    {
        report.push(
            "SCHEMA_VERSION_UNSUPPORTED",
            "$.schema_version",
            "expected water-public-observation-bundle/v0.1",
        );
    }

    let Some(sources) = bundle.get("source_records").and_then(Value::as_array) else {
        report.push(
            "SOURCE_RECORDS_MISSING",
            "$.source_records",
            "source_records must be an array",
        );
        return report;
    };
    let Some(observations) = bundle.get("observations").and_then(Value::as_array) else {
        report.push(
            "OBSERVATIONS_MISSING",
            "$.observations",
            "observations must be an array",
        );
        return report;
    };

    let mut source_ids = HashSet::new();
    for (index, source) in sources.iter().enumerate() {
        let path = format!("$.source_records[{index}].source_id");
        match source.get("source_id").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => {
                if !source_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_SOURCE_ID",
                        path,
                        format!("source id {id:?} occurs more than once"),
                    );
                }
            }
            _ => report.push(
                "SOURCE_ID_MISSING",
                path,
                "source record needs a non-empty source_id",
            ),
        }
    }

    let mut observation_ids = HashSet::new();
    for (index, observation) in observations.iter().enumerate() {
        let path = format!("$.observations[{index}]");
        match observation.get("observation_id").and_then(Value::as_str) {
            Some(id) if !id.trim().is_empty() => {
                if !observation_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_OBSERVATION_ID",
                        format!("{path}.observation_id"),
                        format!("observation id {id:?} occurs more than once"),
                    );
                }
            }
            _ => report.push(
                "OBSERVATION_ID_MISSING",
                format!("{path}.observation_id"),
                "observation needs a non-empty observation_id",
            ),
        }

        match observation.get("source_id").and_then(Value::as_str) {
            Some(id) if source_ids.contains(id) => {}
            Some(id) => report.push(
                "OBSERVATION_SOURCE_UNKNOWN",
                format!("{path}.source_id"),
                format!("source id {id:?} is not present in source_records"),
            ),
            None => report.push(
                "OBSERVATION_SOURCE_MISSING",
                format!("{path}.source_id"),
                "observation requires source_id",
            ),
        }

        match (
            observation.get("period_start"),
            observation.get("period_end"),
        ) {
            (Some(start), Some(end)) if start.is_string() && end.is_string() => {
                let start = start.as_str().unwrap_or_default();
                let end = end.as_str().unwrap_or_default();
                // The schema supplies ISO date format checks; lexicographic order
                // is chronological for canonical YYYY-MM-DD strings.
                if start.len() == 10 && end.len() == 10 && start > end {
                    report.push(
                        "OBSERVATION_PERIOD_REVERSED",
                        path.clone(),
                        format!("period_start {start} is after period_end {end}"),
                    );
                }
            }
            (Some(start), Some(end)) if start.is_null() && end.is_null() => {}
            _ => report.push(
                "OBSERVATION_PERIOD_INCOMPLETE",
                path,
                "period_start and period_end must both be dates or both be null",
            ),
        }

        if observation.get("name").and_then(Value::as_str).is_none_or(str::is_empty) {
            report.push(
                "OBSERVATION_NAME_MISSING",
                format!("{path}.name"),
                "observation name must be a non-empty string",
            );
        }
        if observation.get("unit").and_then(Value::as_str).is_none_or(str::is_empty) {
            report.push(
                "OBSERVATION_UNIT_MISSING",
                format!("{path}.unit"),
                "observation unit must be a non-empty string",
            );
        }
        if observation.get("value").is_none() {
            report.push(
                "OBSERVATION_VALUE_MISSING",
                format!("{path}.value"),
                "observation must preserve the reported value",
            );
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/water-engineering/johannesburg-water-public-observations-2024-25.v0.1.json"
        ))
        .expect("public observation fixture must be valid JSON")
    }

    #[test]
    fn real_public_aggregate_snapshot_has_closed_source_references() {
        let bundle = fixture();
        let report = validate_public_observation_bundle(&bundle);
        assert!(
            report.is_valid(),
            "unexpected semantic issues: {:#?}",
            report.issues
        );
        assert_eq!(bundle["observations"].as_array().map(Vec::len), Some(68));
    }

    #[test]
    fn rejects_unknown_source_reference() {
        let mut bundle = fixture();
        bundle["observations"][0]["source_id"] = json!("missing-source");
        let report = validate_public_observation_bundle(&bundle);
        assert!(report.issues.iter().any(|issue| issue.code == "OBSERVATION_SOURCE_UNKNOWN"));
    }

    #[test]
    fn rejects_reversed_reporting_period() {
        let mut bundle = fixture();
        bundle["observations"][0]["period_start"] = json!("2025-07-01");
        bundle["observations"][0]["period_end"] = json!("2024-07-01");
        let report = validate_public_observation_bundle(&bundle);
        assert!(report.issues.iter().any(|issue| issue.code == "OBSERVATION_PERIOD_REVERSED"));
    }

    #[test]
    fn rejects_duplicate_observation_ids() {
        let mut bundle = fixture();
        let duplicate_id = bundle["observations"][0]["observation_id"].clone();
        bundle["observations"][1]["observation_id"] = duplicate_id;
        let report = validate_public_observation_bundle(&bundle);
        assert!(report.issues.iter().any(|issue| issue.code == "DUPLICATE_OBSERVATION_ID"));
    }

    #[test]
    fn rejects_duplicate_source_ids() {
        let mut bundle = fixture();
        let duplicate_id = bundle["source_records"][0]["source_id"].clone();
        bundle["source_records"][1]["source_id"] = duplicate_id;
        let report = validate_public_observation_bundle(&bundle);
        assert!(report.issues.iter().any(|issue| issue.code == "DUPLICATE_SOURCE_ID"));
    }

    #[test]
    fn annual_report_rows_remain_provisional_until_primary_page_review() {
        let bundle = fixture();
        let observations = bundle["observations"]
            .as_array()
            .expect("observations must be an array");
        let mut annual_report_rows = 0usize;
        for observation in observations {
            let source_id = observation["source_id"].as_str().unwrap_or_default();
            if source_id == "jhb-water-annual-report-2023-24"
                || source_id == "jhb-water-annual-report-2024-25"
            {
                annual_report_rows += 1;
                assert_eq!(
                    observation["interpretation_state"].as_str(),
                    Some("provisional_needs_primary_document_check"),
                    "annual report row must not be promoted before page-level review: {}",
                    observation["observation_id"]
                );
            }
        }
        assert_eq!(annual_report_rows, 44);
    }
}
