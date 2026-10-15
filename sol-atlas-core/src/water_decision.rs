// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Fail-closed semantic consistency checks for water intervention decision packets.
//!
//! JSON Schema validation is a separate gate. These checks do not resolve or
//! authenticate referenced run bundles, compute Pareto frontiers, calibrate a
//! model, or confer engineering approval or operational authority.

use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionIssue {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecisionValidationReport {
    pub issues: Vec<DecisionIssue>,
}

impl DecisionValidationReport {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    fn push(
        &mut self,
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.issues.push(DecisionIssue {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

#[derive(Debug, Clone)]
struct MetricDescriptor {
    metric_id: String,
    case_id: String,
    unit: String,
    scope: String,
    period_label: String,
    availability_state: String,
    comparability_state: String,
}

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn is_nonempty_text(value: Option<&str>) -> bool {
    value.is_some_and(|text| !text.trim().is_empty())
}

fn is_sha256(value: Option<&str>) -> bool {
    value.is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

/// Validate packet-local consistency. Call JSON Schema validation separately.
///
/// A valid report means the packet is internally consistent under these rules;
/// it does not mean referenced files exist, evidence is authentic, or a model
/// is accurate enough for a real-world engineering decision.
pub fn validate_decision_packet(packet: &Value) -> DecisionValidationReport {
    let mut report = DecisionValidationReport::default();
    if !packet.is_object() {
        report.push("PACKET_NOT_OBJECT", "$", "decision packet must be a JSON object");
        return report;
    }

    if text_at(packet, "schema_version") != Some("water-intervention-decision/v0.1") {
        report.push(
            "SCHEMA_VERSION_UNSUPPORTED",
            "$.schema_version",
            "expected water-intervention-decision/v0.1",
        );
    }

    let Some(baseline) = packet.get("baseline") else {
        report.push("BASELINE_MISSING", "$.baseline", "baseline is required");
        return report;
    };
    let Some(alternatives) = packet.get("alternatives").and_then(Value::as_array) else {
        report.push(
            "ALTERNATIVES_MISSING",
            "$.alternatives",
            "alternatives must be an array",
        );
        return report;
    };

    let mut cases: Vec<(&Value, bool)> = Vec::with_capacity(alternatives.len() + 1);
    cases.push((baseline, true));
    for alternative in alternatives {
        cases.push((alternative, false));
    }

    let mut case_ids = HashSet::new();
    let mut scenario_ids = HashSet::new();
    let mut alternative_ids = HashSet::new();
    let mut descriptors = Vec::new();

    for (case_index, (case, is_baseline)) in cases.iter().enumerate() {
        let case_path = if *is_baseline {
            "$.baseline".to_owned()
        } else {
            format!("$.alternatives[{}]", case_index - 1)
        };
        let id_field = if *is_baseline {
            "case_id"
        } else {
            "alternative_id"
        };
        match text_at(case, id_field) {
            Some(id) if !id.trim().is_empty() => {
                if !case_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_CASE_ID",
                        format!("{case_path}.{id_field}"),
                        format!("case identifier {id:?} is duplicated"),
                    );
                }
                if !*is_baseline && !alternative_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_ALTERNATIVE_ID",
                        format!("{case_path}.{id_field}"),
                        format!("alternative identifier {id:?} is duplicated"),
                    );
                }
            }
            _ => report.push(
                "CASE_ID_MISSING",
                format!("{case_path}.{id_field}"),
                "case requires a non-empty identifier",
            ),
        }

        let this_case_id = text_at(case, id_field).unwrap_or("<missing>").to_owned();
        let run_state = case
            .get("run_reference")
            .and_then(|run| text_at(run, "run_state"));
        match text_at(case, "scenario_id") {
            Some(id) if !id.trim().is_empty() => {
                if !scenario_ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_SCENARIO_ID",
                        format!("{case_path}.scenario_id"),
                        format!("scenario identifier {id:?} is repeated across cases"),
                    );
                }
            }
            _ => report.push(
                "SCENARIO_ID_MISSING",
                format!("{case_path}.scenario_id"),
                "case requires a non-empty scenario identifier",
            ),
        }

        validate_run_reference(case, &case_path, &mut report);

        let Some(metrics) = case.get("metrics").and_then(Value::as_array) else {
            report.push(
                "CASE_METRICS_MISSING",
                format!("{case_path}.metrics"),
                "metrics must be an array; missing metrics must not be replaced with zero",
            );
            validate_constraints(case, &case_path, &mut report);
            continue;
        };
        let mut metric_ids = HashSet::new();
        for (metric_index, metric) in metrics.iter().enumerate() {
            let metric_path = format!("{case_path}.metrics[{metric_index}]");
            let descriptor = validate_metric(
                metric,
                &metric_path,
                &this_case_id,
                run_state,
                &mut metric_ids,
                &mut report,
            );
            if let Some(descriptor) = descriptor {
                descriptors.push(descriptor);
            }
        }

        validate_constraints(case, &case_path, &mut report);
    }

    validate_comparable_metric_dimensions(&descriptors, &mut report);
    validate_comparison_policy(packet, &mut report);
    validate_decision_state(packet, baseline, alternatives, &mut report);
    report
}

fn validate_run_reference(
    case: &Value,
    case_path: &str,
    report: &mut DecisionValidationReport,
) {
    let path = format!("{case_path}.run_reference");
    let Some(run) = case.get("run_reference") else {
        report.push("RUN_REFERENCE_MISSING", path, "run_reference is required");
        return;
    };

    let state = text_at(run, "run_state");
    let digest = text_at(run, "manifest_sha256");
    let ceiling = text_at(run, "evidence_ceiling");

    if !is_nonempty_text(text_at(run, "bundle_id")) {
        report.push(
            "RUN_BUNDLE_ID_MISSING",
            format!("{path}.bundle_id"),
            "run reference requires a non-empty bundle_id",
        );
    }

    match state {
        Some("NotRun") => {
            if !run
                .get("manifest_sha256")
                .is_some_and(Value::is_null)
                || ceiling != Some("contract_only")
            {
                report.push(
                    "NOT_RUN_EVIDENCE_PROMOTION",
                    path,
                    "NotRun must have a null manifest digest and contract_only evidence ceiling",
                );
            }
        }
        Some("Completed") => {
            if !is_sha256(digest) {
                report.push(
                    "COMPLETED_RUN_DIGEST_INVALID",
                    format!("{path}.manifest_sha256"),
                    "Completed run reference requires a lowercase 64-hex manifest SHA-256",
                );
            }
            if !matches!(
                ceiling,
                Some("synthetic_unvalidated" | "synthetic_reproduced" | "empirically_validated")
            ) {
                report.push(
                    "COMPLETED_RUN_EVIDENCE_CEILING_INVALID",
                    format!("{path}.evidence_ceiling"),
                    "Completed run cannot retain the contract_only evidence ceiling",
                );
            }
        }
        Some("Failed" | "Blocked") => {
            if matches!(ceiling, Some("synthetic_reproduced" | "empirically_validated")) {
                report.push(
                    "FAILED_RUN_EVIDENCE_PROMOTION",
                    format!("{path}.evidence_ceiling"),
                    "Failed or Blocked runs cannot claim reproduced or empirically validated evidence",
                );
            }
        }
        _ => report.push(
            "RUN_STATE_INVALID",
            format!("{path}.run_state"),
            "run_state must be NotRun, Completed, Failed, or Blocked",
        ),
    }
}

fn validate_metric(
    metric: &Value,
    path: &str,
    case_id: &str,
    run_state: Option<&str>,
    metric_ids: &mut HashSet<String>,
    report: &mut DecisionValidationReport,
) -> Option<MetricDescriptor> {
    let metric_id = text_at(metric, "metric_id");
    match metric_id {
        Some(id) if !id.trim().is_empty() => {
            if !metric_ids.insert(id.to_owned()) {
                report.push(
                    "DUPLICATE_METRIC_ID_WITHIN_CASE",
                    format!("{path}.metric_id"),
                    format!("metric identifier {id:?} is repeated within this case"),
                );
            }
        }
        _ => report.push(
            "METRIC_ID_MISSING",
            format!("{path}.metric_id"),
            "metric requires a non-empty metric_id",
        ),
    }

    let availability = text_at(metric, "availability_state");
    let evidence = text_at(metric, "evidence_class");
    let reason = metric.get("reason");
    let value = metric.get("value");
    let sources = metric.get("source_references").and_then(Value::as_array);

    match availability {
        Some("available") => {
            if value.and_then(Value::as_f64).is_none() {
                report.push(
                    "AVAILABLE_METRIC_VALUE_MISSING",
                    format!("{path}.value"),
                    "available metric requires a numeric value",
                );
            }
            if !reason.is_some_and(Value::is_null) {
                report.push(
                    "AVAILABLE_METRIC_REASON_CONFLICT",
                    format!("{path}.reason"),
                    "available metric must use reason=null",
                );
            }
            if evidence == Some("unavailable") {
                report.push(
                    "AVAILABLE_METRIC_EVIDENCE_CONFLICT",
                    format!("{path}.evidence_class"),
                    "available metric cannot use evidence_class=unavailable",
                );
            }
            if !sources.is_some_and(|items| {
                !items.is_empty()
                    && items
                        .iter()
                        .all(|item| item.as_str().is_some_and(|text| !text.trim().is_empty()))
            }) {
                report.push(
                    "AVAILABLE_METRIC_SOURCE_MISSING",
                    format!("{path}.source_references"),
                    "available metrics require at least one non-empty evidence reference",
                );
            }
        }
        Some("unavailable" | "not_comparable") => {
            if !value.is_some_and(Value::is_null) {
                report.push(
                    "UNAVAILABLE_METRIC_HAS_VALUE",
                    format!("{path}.value"),
                    "unavailable or non-comparable metric must have value=null, never a substituted zero",
                );
            }
            if !reason
                .and_then(Value::as_str)
                .is_some_and(|text| !text.trim().is_empty())
            {
                report.push(
                    "UNAVAILABLE_METRIC_REASON_MISSING",
                    format!("{path}.reason"),
                    "unavailable or non-comparable metric requires a non-empty reason",
                );
            }
            if availability == Some("unavailable") && evidence != Some("unavailable") {
                report.push(
                    "UNAVAILABLE_METRIC_EVIDENCE_CONFLICT",
                    format!("{path}.evidence_class"),
                    "unavailable metric must use evidence_class=unavailable",
                );
            }
        }
        _ => report.push(
            "METRIC_AVAILABILITY_INVALID",
            format!("{path}.availability_state"),
            "availability_state must be available, unavailable, or not_comparable",
        ),
    }

    if availability == Some("available") && run_state != Some("Completed") {
        report.push(
            "METRIC_WITHOUT_COMPLETED_RUN",
            format!("{path}.availability_state"),
            "numeric/available metrics require a Completed run; NotRun, Failed, and Blocked cases cannot supply accepted metric values",
        );
    }

    let comparability = text_at(metric, "comparability_state");
    if comparability == Some("comparable") && availability != Some("available") {
        report.push(
            "UNAVAILABLE_METRIC_MARKED_COMPARABLE",
            format!("{path}.comparability_state"),
            "only an available metric can be marked comparable",
        );
    }

    validate_uncertainty(metric.get("uncertainty"), &format!("{path}.uncertainty"), report);

    let descriptor_fields = (
        metric_id,
        text_at(metric, "unit"),
        text_at(metric, "scope"),
        text_at(metric, "period_label"),
        availability,
        comparability,
    );
    match descriptor_fields {
        (Some(id), Some(unit), Some(scope), Some(period), Some(availability), Some(comparability))
            if !id.trim().is_empty()
                && !unit.trim().is_empty()
                && !scope.trim().is_empty()
                && !period.trim().is_empty() =>
        {
            Some(MetricDescriptor {
                metric_id: id.to_owned(),
                case_id: case_id.to_owned(),
                unit: unit.to_owned(),
                scope: scope.to_owned(),
                period_label: period.to_owned(),
                availability_state: availability.to_owned(),
                comparability_state: comparability.to_owned(),
            })
        }
        _ => {
            report.push(
                "METRIC_COMPARISON_METADATA_MISSING",
                path.to_owned(),
                "metric requires non-empty id, unit, scope, period and explicit comparability state",
            );
            None
        }
    }
}

fn validate_uncertainty(
    uncertainty: Option<&Value>,
    path: &str,
    report: &mut DecisionValidationReport,
) {
    let Some(uncertainty) = uncertainty else {
        report.push("UNCERTAINTY_MISSING", path, "uncertainty object is required");
        return;
    };
    let state = text_at(uncertainty, "state");
    let lower = uncertainty.get("lower_bound");
    let upper = uncertainty.get("upper_bound");

    match state {
        Some("not_assessed" | "point_estimate_only") => {
            if !lower.is_some_and(Value::is_null) || !upper.is_some_and(Value::is_null) {
                report.push(
                    "UNCERTAINTY_BOUNDS_CONFLICT",
                    path,
                    "not_assessed and point_estimate_only must not imply numeric interval bounds",
                );
            }
        }
        Some("interval" | "scenario_sweep" | "sensitivity_analysis") => {
            let bounds = (lower.and_then(Value::as_f64), upper.and_then(Value::as_f64));
            match bounds {
                (Some(low), Some(high)) if low <= high => {}
                _ => report.push(
                    "UNCERTAINTY_BOUNDS_INVALID",
                    path,
                    "interval/sweep/sensitivity uncertainty needs numeric lower_bound <= upper_bound",
                ),
            }
        }
        _ => report.push(
            "UNCERTAINTY_STATE_INVALID",
            format!("{path}.state"),
            "uncertainty state is missing or unsupported",
        ),
    }
}

fn validate_constraints(case: &Value, case_path: &str, report: &mut DecisionValidationReport) {
    let path = format!("{case_path}.hard_constraints");
    let Some(constraints) = case.get("hard_constraints").and_then(Value::as_array) else {
        report.push(
            "HARD_CONSTRAINTS_MISSING",
            path,
            "hard_constraints must be an array",
        );
        return;
    };
    if constraints.is_empty() {
        report.push(
            "HARD_CONSTRAINTS_EMPTY",
            path,
            "mandatory service/safety constraints must be explicit even when their state is unknown",
        );
        return;
    }

    let mut ids = HashSet::new();
    for (index, constraint) in constraints.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        match text_at(constraint, "constraint_id") {
            Some(id) if !id.trim().is_empty() => {
                if !ids.insert(id.to_owned()) {
                    report.push(
                        "DUPLICATE_CONSTRAINT_ID_WITHIN_CASE",
                        format!("{item_path}.constraint_id"),
                        format!("constraint identifier {id:?} is repeated within this case"),
                    );
                }
            }
            _ => report.push(
                "CONSTRAINT_ID_MISSING",
                format!("{item_path}.constraint_id"),
                "constraint requires a non-empty identifier",
            ),
        }

        let status = text_at(constraint, "status");
        let basis = text_at(constraint, "validation_basis");
        let reference = text_at(constraint, "evidence_reference");
        match status {
            Some("unknown") => {
                if !constraint
                    .get("evidence_reference")
                    .is_some_and(Value::is_null)
                {
                    report.push(
                        "UNKNOWN_CONSTRAINT_HAS_EVIDENCE",
                        format!("{item_path}.evidence_reference"),
                        "unknown constraint status must not claim a passing/failing evidence reference",
                    );
                }
            }
            Some("pass" | "fail") => {
                if !matches!(
                    basis,
                    Some("regulatory_or_contractual" | "engineer_defined")
                ) || !is_nonempty_text(reference)
                {
                    report.push(
                        "CONSTRAINT_RESULT_UNGROUNDED",
                        item_path,
                        "pass/fail requires a defined regulatory/contractual or engineer-defined basis and a non-empty evidence reference",
                    );
                }
            }
            _ => report.push(
                "CONSTRAINT_STATUS_INVALID",
                format!("{item_path}.status"),
                "constraint status must be pass, fail, or unknown",
            ),
        }
    }
}

fn validate_comparable_metric_dimensions(
    descriptors: &[MetricDescriptor],
    report: &mut DecisionValidationReport,
) {
    for (index, left) in descriptors.iter().enumerate() {
        for right in descriptors.iter().skip(index + 1) {
            if left.metric_id != right.metric_id || left.case_id == right.case_id {
                continue;
            }
            if left.comparability_state != "comparable"
                && right.comparability_state != "comparable"
            {
                continue;
            }
            let path = format!("metric[{}]", left.metric_id);
            if left.comparability_state != "comparable"
                || right.comparability_state != "comparable"
            {
                report.push(
                    "COMPARABILITY_STATE_MISMATCH",
                    path.clone(),
                    "a metric must be explicitly comparable in both cases or retained separately",
                );
            }
            if left.availability_state == "available"
                && right.availability_state == "available"
            {
                if left.unit != right.unit {
                    report.push(
                        "COMPARABLE_METRIC_UNIT_MISMATCH",
                        path.clone(),
                        format!(
                            "comparable metric uses units {:?} and {:?}; explicit normalisation is required before comparison",
                            left.unit, right.unit
                        ),
                    );
                }
                if left.scope != right.scope {
                    report.push(
                        "COMPARABLE_METRIC_SCOPE_MISMATCH",
                        path.clone(),
                        format!(
                            "comparable metric uses scopes {:?} and {:?}",
                            left.scope, right.scope
                        ),
                    );
                }
                if left.period_label != right.period_label {
                    report.push(
                        "COMPARABLE_METRIC_PERIOD_MISMATCH",
                        path,
                        format!(
                            "comparable metric uses periods {:?} and {:?}",
                            left.period_label, right.period_label
                        ),
                    );
                }
            }
        }
    }
}

fn validate_comparison_policy(packet: &Value, report: &mut DecisionValidationReport) {
    let policy = packet.get("comparison_policy");
    let expected = [
        ("method", "pareto_frontier"),
        ("unknown_metric_behavior", "not_zero_and_not_rankable"),
        ("non_comparable_metric_behavior", "retain_separately"),
    ];
    for (field, value) in expected {
        if text_at(policy.unwrap_or(&Value::Null), field) != Some(value) {
            report.push(
                "COMPARISON_POLICY_UNSAFE",
                format!("$.comparison_policy.{field}"),
                format!("comparison policy must set {field}={value}"),
            );
        }
    }
    if !policy
        .and_then(|value| value.get("hard_constraints_first"))
        .is_some_and(|value| value == &Value::Bool(true))
    {
        report.push(
            "COMPARISON_POLICY_UNSAFE",
            "$.comparison_policy.hard_constraints_first",
            "hard constraints must precede any trade-off analysis",
        );
    }
    if !policy
        .and_then(|value| value.get("scalar_score_allowed"))
        .is_some_and(|value| value == &Value::Bool(false))
    {
        report.push(
            "COMPARISON_POLICY_UNSAFE",
            "$.comparison_policy.scalar_score_allowed",
            "implicit scalar scores are forbidden in this contract version",
        );
    }
}

fn candidate_constraints_pass(candidate: &Value) -> bool {
    let Some(constraints) = candidate.get("hard_constraints").and_then(Value::as_array) else {
        return false;
    };
    !constraints.is_empty()
        && constraints.iter().all(|constraint| {
            text_at(constraint, "status") == Some("pass")
                && matches!(
                    text_at(constraint, "validation_basis"),
                    Some("regulatory_or_contractual" | "engineer_defined")
                )
                && is_nonempty_text(text_at(constraint, "evidence_reference"))
        })
}

fn has_comparable_metric_pair(baseline: &Value, candidate: &Value) -> bool {
    let baseline_metrics = baseline
        .get("metrics")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let candidate_metrics = candidate
        .get("metrics")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    baseline_metrics.iter().any(|left| {
        candidate_metrics.iter().any(|right| {
            text_at(left, "metric_id") == text_at(right, "metric_id")
                && text_at(left, "availability_state") == Some("available")
                && text_at(right, "availability_state") == Some("available")
                && text_at(left, "comparability_state") == Some("comparable")
                && text_at(right, "comparability_state") == Some("comparable")
                && text_at(left, "unit") == text_at(right, "unit")
                && text_at(left, "scope") == text_at(right, "scope")
                && text_at(left, "period_label") == text_at(right, "period_label")
                && left.get("value").and_then(Value::as_f64).is_some()
                && right.get("value").and_then(Value::as_f64).is_some()
        })
    })
}

fn validate_decision_state(
    packet: &Value,
    baseline: &Value,
    alternatives: &[Value],
    report: &mut DecisionValidationReport,
) {
    let decision = packet.get("decision_state");
    let state = text_at(decision.unwrap_or(&Value::Null), "state");
    let selected = text_at(decision.unwrap_or(&Value::Null), "selected_alternative_id");
    let authorization = text_at(decision.unwrap_or(&Value::Null), "authorization_state");

    if !matches!(authorization, Some("not_authorized" | "planning_only")) {
        report.push(
            "AUTHORIZATION_STATE_FORBIDDEN",
            "$.decision_state.authorization_state",
            "this contract cannot authorize physical/field operations; use not_authorized or planning_only",
        );
    }

    if state != Some("recorded") {
        if selected.is_some() {
            report.push(
                "UNRECORDED_DECISION_HAS_SELECTION",
                "$.decision_state.selected_alternative_id",
                "selected_alternative_id must be null until a decision is recorded",
            );
        }
        if !matches!(state, Some("not_decided" | "review_required")) {
            report.push(
                "DECISION_STATE_INVALID",
                "$.decision_state.state",
                "decision state must be not_decided, review_required, or recorded",
            );
        }
        return;
    }

    let Some(selected_id) = selected else {
        report.push(
            "RECORDED_DECISION_SELECTION_MISSING",
            "$.decision_state.selected_alternative_id",
            "recorded decision must identify an alternative",
        );
        return;
    };
    let Some(candidate) = alternatives
        .iter()
        .find(|candidate| text_at(candidate, "alternative_id") == Some(selected_id))
    else {
        report.push(
            "SELECTED_ALTERNATIVE_UNKNOWN",
            "$.decision_state.selected_alternative_id",
            format!("selected alternative {selected_id:?} is not present in alternatives"),
        );
        return;
    };

    let Some(run) = candidate.get("run_reference") else {
        report.push(
            "RUN_REFERENCE_MISSING",
            "$.decision_state.selected_alternative_id",
            "selected alternative has no run_reference",
        );
        return;
    };
    if text_at(run, "run_state") != Some("Completed") {
        report.push(
            "SELECTED_ALTERNATIVE_NOT_RUN",
            "$.decision_state.selected_alternative_id",
            "a selected alternative must reference a Completed run",
        );
    }
    if !matches!(
        text_at(run, "evidence_ceiling"),
        Some("synthetic_reproduced" | "empirically_validated")
    ) {
        report.push(
            "SELECTED_ALTERNATIVE_EVIDENCE_INSUFFICIENT",
            "$.decision_state.selected_alternative_id",
            "a recorded planning decision requires reproduced synthetic or empirically validated run evidence",
        );
    }
    let baseline_run = baseline.get("run_reference").unwrap_or(&Value::Null);
    if text_at(baseline_run, "run_state") != Some("Completed") {
        report.push(
            "BASELINE_NOT_RUN",
            "$.baseline.run_reference.run_state",
            "a recorded intervention decision requires a Completed baseline run",
        );
    }
    if !matches!(
        text_at(baseline_run, "evidence_ceiling"),
        Some("synthetic_reproduced" | "empirically_validated")
    ) {
        report.push(
            "BASELINE_EVIDENCE_INSUFFICIENT",
            "$.baseline.run_reference.evidence_ceiling",
            "a recorded intervention decision requires reproduced synthetic or empirically validated baseline evidence",
        );
    }

    if !candidate_constraints_pass(candidate) {
        report.push(
            "SELECTED_ALTERNATIVE_CONSTRAINTS_NOT_PASSED",
            "$.decision_state.selected_alternative_id",
            "every hard constraint must pass against a defined basis with an evidence reference before an alternative can be selected",
        );
    }
    if !has_comparable_metric_pair(baseline, candidate) {
        report.push(
            "SELECTED_ALTERNATIVE_NO_COMPARABLE_OUTCOME",
            "$.decision_state.selected_alternative_id",
            "selection requires at least one available, explicitly comparable outcome in both baseline and candidate",
        );
    }

    let uncertainty = packet.get("uncertainty_review").unwrap_or(&Value::Null);
    if text_at(uncertainty, "state") != Some("assessed")
        || text_at(uncertainty, "sensitivity_state") != Some("ran")
    {
        report.push(
            "DECISION_UNCERTAINTY_GATE_UNMET",
            "$.uncertainty_review",
            "recorded decisions require an assessed uncertainty review and a completed sensitivity analysis",
        );
    }

    let equity = packet.get("equity_review").unwrap_or(&Value::Null);
    if text_at(equity, "state") != Some("assessed") {
        report.push(
            "DECISION_EQUITY_GATE_UNMET",
            "$.equity_review",
            "recorded decisions require an explicit equity/service-distribution review",
        );
    }
    if text_at(decision.unwrap_or(&Value::Null), "independent_engineer_review") != Some("completed")
    {
        report.push(
            "DECISION_INDEPENDENT_REVIEW_MISSING",
            "$.decision_state.independent_engineer_review",
            "recorded decisions require completed independent engineering review",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../fixtures/water-engineering/w0-intervention-decision.synthetic.json"
        ))
        .expect("water intervention decision fixture must be valid JSON")
    }

    fn qualify_case(case: &mut Value) {
        let bundle_id = case["run_reference"]["bundle_id"]
            .as_str()
            .expect("fixture has bundle id")
            .to_owned();
        case["run_reference"]["run_state"] = json!("Completed");
        case["run_reference"]["manifest_sha256"] = json!("a".repeat(64));
        case["run_reference"]["evidence_ceiling"] = json!("synthetic_reproduced");

        for metric in case["metrics"]
            .as_array_mut()
            .expect("fixture metrics array")
        {
            metric["availability_state"] = json!("available");
            metric["value"] = json!(1.0);
            metric["evidence_class"] = json!("simulated");
            metric["comparability_state"] = json!("comparable");
            metric["scope"] = json!("synthetic W0 service envelope");
            metric["period_label"] = json!("synthetic steady-state interval");
            metric["source_references"] = json!([bundle_id.clone()]);
            metric["reason"] = Value::Null;
            metric["uncertainty"] = json!({
                "state":"point_estimate_only",
                "lower_bound":null,
                "upper_bound":null,
                "method_note":"Only the deterministic synthetic reference case was evaluated."
            });
        }

        for constraint in case["hard_constraints"]
            .as_array_mut()
            .expect("fixture hard constraints array")
        {
            constraint["status"] = json!("pass");
            constraint["validation_basis"] = json!("engineer_defined");
            constraint["evidence_reference"] = json!("engineering-review:synthetic-capture-1");
        }
    }

    #[test]
    fn explicit_not_run_fixture_is_internally_consistent() {
        let report = validate_decision_packet(&fixture());
        assert!(
            report.is_valid(),
            "unexpected issues: {:#?}",
            report.issues
        );
    }

    #[test]
    fn unavailable_metric_cannot_smuggle_in_a_zero() {
        let mut packet = fixture();
        packet["baseline"]["metrics"][0]["value"] = json!(0.0);
        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "UNAVAILABLE_METRIC_HAS_VALUE"));
    }

    #[test]
    fn not_run_case_cannot_claim_completed_evidence() {
        let mut packet = fixture();
        packet["baseline"]["run_reference"]["evidence_ceiling"] =
            json!("synthetic_reproduced");
        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "NOT_RUN_EVIDENCE_PROMOTION"));
    }

    #[test]
    fn not_run_case_cannot_claim_available_metric() {
        let mut packet = fixture();
        let metric = &mut packet["baseline"]["metrics"][0];
        metric["availability_state"] = json!("available");
        metric["value"] = json!(0.0);
        metric["evidence_class"] = json!("simulated");
        metric["comparability_state"] = json!("comparable");
        metric["reason"] = Value::Null;
        metric["source_references"] = json!(["unverified-run-reference"]);

        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "METRIC_WITHOUT_COMPLETED_RUN"));
    }

    #[test]
    fn recorded_decision_requires_a_completed_baseline() {
        let mut packet = fixture();
        qualify_case(&mut packet["alternatives"][0]);
        packet["decision_state"]["state"] = json!("recorded");
        let selected = packet["alternatives"][0]["alternative_id"].clone();
        packet["decision_state"]["selected_alternative_id"] = selected;
        packet["uncertainty_review"]["state"] = json!("assessed");
        packet["uncertainty_review"]["sensitivity_state"] = json!("ran");
        packet["equity_review"]["state"] = json!("assessed");
        packet["decision_state"]["authorization_state"] = json!("planning_only");
        packet["decision_state"]["independent_engineer_review"] = json!("completed");

        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "BASELINE_NOT_RUN"));
    }

    #[test]
    fn duplicate_alternative_ids_are_rejected() {
        let mut packet = fixture();
        let duplicate = packet["alternatives"][0].clone();
        packet["alternatives"]
            .as_array_mut()
            .expect("alternatives array")
            .push(duplicate);
        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "DUPLICATE_CASE_ID"));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "DUPLICATE_SCENARIO_ID"));
    }

    #[test]
    fn comparable_metrics_must_share_unit_scope_and_period() {
        let mut packet = fixture();
        qualify_case(&mut packet["baseline"]);
        qualify_case(&mut packet["alternatives"][0]);
        packet["alternatives"][0]["metrics"][0]["unit"] = json!("litre/second");
        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "COMPARABLE_METRIC_UNIT_MISMATCH"));
    }

    #[test]
    fn recorded_decision_is_blocked_when_constraints_are_unknown() {
        let mut packet = fixture();
        packet["decision_state"]["state"] = json!("recorded");
        let selected = packet["alternatives"][0]["alternative_id"].clone();
        packet["decision_state"]["selected_alternative_id"] = selected;
        let report = validate_decision_packet(&packet);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "SELECTED_ALTERNATIVE_NOT_RUN"));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "SELECTED_ALTERNATIVE_CONSTRAINTS_NOT_PASSED"));
    }

    #[test]
    fn qualified_planning_record_requires_all_declared_gates() {
        let mut packet = fixture();
        qualify_case(&mut packet["baseline"]);
        qualify_case(&mut packet["alternatives"][0]);
        packet["uncertainty_review"]["state"] = json!("assessed");
        packet["uncertainty_review"]["sensitivity_state"] = json!("ran");
        packet["uncertainty_review"]["material_unknowns"] = json!([
            "No unresolved material unknown remains within the synthetic scope; residual limitations are recorded in nonclaims."
        ]);
        for action in packet["uncertainty_review"]["next_evidence_actions"]
            .as_array_mut()
            .expect("next evidence actions")
        {
            action["blocking"] = json!(false);
        }
        packet["equity_review"]["state"] = json!("assessed");
        packet["equity_review"]["unresolved_impacts"] = json!([
            "Real-world population effects are outside this synthetic fixture and require a separate review before transfer."
        ]);
        packet["decision_state"]["state"] = json!("recorded");
        let selected = packet["alternatives"][0]["alternative_id"].clone();
        packet["decision_state"]["selected_alternative_id"] = selected;
        packet["decision_state"]["authorization_state"] = json!("planning_only");
        packet["decision_state"]["independent_engineer_review"] = json!("completed");

        let report = validate_decision_packet(&packet);
        assert!(
            report.is_valid(),
            "unexpected issues: {:#?}",
            report.issues
        );
    }
}
