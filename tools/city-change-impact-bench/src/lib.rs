//! Read-only CityMaker projection of owner-qualified change-impact reports.
//!
//! This adapter does not compute the authoritative dependency graph or impact
//! frontier. It validates exact report bindings and projects owner findings to
//! city claims without increasing their authority or claim ceiling.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

const CLAIM_CEILING: &str = "synthetic owner-report projection semantics only; no real dependency coverage, change authority, engineering applicability or action authority is qualified";

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn records<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn string_array(value: &Value, key: &str) -> BTreeSet<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn contains_edge(graph: &Value, from: &str, to: &str) -> bool {
    records(graph, "edges").iter().any(|edge| {
        text(edge, "from") == from && text(edge, "to") == to && !text(edge, "relation").is_empty()
    })
}

fn graph_has_cycle(graph: &Value) -> bool {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in records(graph, "edges") {
        let from = text(edge, "from");
        let to = text(edge, "to");
        if from.is_empty() || to.is_empty() {
            return true;
        }
        adjacency
            .entry(from.to_owned())
            .or_default()
            .push(to.to_owned());
    }

    fn visit(
        node: &str,
        adjacency: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node.to_owned());
        if let Some(next) = adjacency.get(node) {
            for neighbor in next {
                if visit(neighbor, adjacency, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_owned());
        false
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    adjacency
        .keys()
        .any(|node| visit(node, &adjacency, &mut visiting, &mut visited))
}

fn expected_reason(finding: &Value) -> Vec<String> {
    let values = string_array(finding, "reason_codes");
    values.into_iter().collect()
}

fn projection(claim_id: &str, state: &str, reason_codes: Vec<String>) -> Value {
    json!({
        "claim_id": claim_id,
        "state": state,
        "reason_codes": reason_codes
    })
}

fn finding_for<'a>(report: &'a Value, claim_id: &str) -> Option<&'a Value> {
    records(report, "claim_findings")
        .iter()
        .find(|finding| text(finding, "claim_id") == claim_id)
}

fn revalidation_for<'a>(input: &'a Value, claim_id: &str) -> Option<&'a Value> {
    records(input, "revalidation_receipts")
        .iter()
        .find(|receipt| text(receipt, "claim_id") == claim_id)
}

fn receipt_is_exact(
    receipt: &Value,
    claim: &Value,
    event: &Value,
    query: &Value,
    graph: &Value,
    report: &Value,
) -> bool {
    let trusted_at = number(receipt, "trusted_at");
    let evaluation_at = number(query, "evaluation_at");
    text(receipt, "state") == "qualified"
        && text(receipt, "evidence_state") == "current"
        && text(receipt, "event_id") == text(event, "event_id")
        && text(receipt, "report_id") == text(report, "report_id")
        && text(receipt, "graph_snapshot_digest") == text(graph, "snapshot_digest")
        && text(receipt, "subject_revision") == text(event, "source_revision_after")
        && text(receipt, "scope_id") == text(claim, "scope_id")
        && text(receipt, "scope_id") == text(query, "scope_id")
        && text(receipt, "scope_digest") == text(claim, "scope_digest")
        && text(receipt, "scope_digest") == text(query, "scope_digest")
        && trusted_at.is_some()
        && evaluation_at.is_some()
        && trusted_at.is_some_and(|trusted| evaluation_at.is_some_and(|at| trusted <= at))
        && !text(receipt, "receipt_id").is_empty()
}

fn evaluate_claim(
    input: &Value,
    claim: &Value,
    event: &Value,
    query: &Value,
    graph: &Value,
    report: &Value,
) -> Value {
    let claim_id = text(claim, "claim_id");
    let finding = finding_for(report, claim_id);
    let mut reasons = BTreeSet::<String>::new();

    if claim_id.is_empty()
        || text(claim, "scope_id") != text(query, "scope_id")
        || text(claim, "scope_digest") != text(query, "scope_digest")
        || !is_digest(text(query, "scope_digest"))
        || text(claim, "functional_unit") != text(query, "functional_unit")
    {
        reasons.insert("CLAIM_SCOPE_OR_FUNCTIONAL_UNIT_MISMATCH".to_owned());
        return projection(
            claim_id,
            "insufficient_evidence",
            reasons.into_iter().collect(),
        );
    }

    let Some(finding) = finding else {
        reasons.insert("CLAIM_IMPACT_FINDING_MISSING".to_owned());
        return projection(
            claim_id,
            "insufficient_evidence",
            reasons.into_iter().collect(),
        );
    };
    let mut finding_reasons = expected_reason(finding);
    if finding_reasons.is_empty() {
        reasons.insert("OWNER_IMPACT_REASON_MISSING".to_owned());
        return projection(
            claim_id,
            "insufficient_evidence",
            reasons.into_iter().collect(),
        );
    }

    if text(event, "change_class") == "coordinate_frame"
        && text(event, "transform_state") == "ambiguous"
    {
        return projection(
            claim_id,
            "insufficient_evidence",
            vec![
                "COORDINATE_TRANSFORM_AMBIGUOUS".to_owned(),
                "GRAPH_COVERAGE_INCOMPLETE".to_owned(),
            ],
        );
    }

    // The change must be authorized by its owner and registered by the
    // declared trust/receipt time before a city projection may rely on it.
    if text(event, "authority_state") != "authorized"
        || number(event, "trusted_at").is_none()
        || number(event, "trusted_at").is_some_and(|trusted| {
            number(query, "evaluation_at").is_none_or(|evaluation| trusted > evaluation)
        })
    {
        if !finding_reasons.contains(&"CHANGE_AUTHORITY_UNVERIFIED".to_owned())
            && !finding_reasons.contains(&"REVOCATION_AUTHORITY_UNVERIFIED".to_owned())
            && !finding_reasons.contains(&"ISSUER_AUTHORITY_UNQUALIFIED".to_owned())
        {
            finding_reasons = vec![
                match text(event, "change_class") {
                    "lot_revocation" => "REVOCATION_AUTHORITY_UNVERIFIED",
                    "payload_retrieval" => "ISSUER_AUTHORITY_UNQUALIFIED",
                    _ => "CHANGE_AUTHORITY_UNVERIFIED",
                }
                .to_owned(),
            ];
        }
        return projection(claim_id, "insufficient_evidence", finding_reasons);
    }

    if text(event, "time_basis") != "UTC_UNIX_SECONDS"
        || number(event, "effective_at").is_none()
        || number(event, "recorded_at").is_none()
        || number(query, "evaluation_at").is_none()
    {
        reasons.insert("CHANGE_EVENT_TIME_BINDING_INVALID".to_owned());
        return projection(claim_id, "invalid_evidence", reasons.into_iter().collect());
    }

    if text(event, "change_class") == "payload_digest_change"
        && text(event, "payload_bytes_state") != "verified"
    {
        return projection(
            claim_id,
            "insufficient_evidence",
            vec!["PAYLOAD_BYTES_NOT_REVERIFIED".to_owned()],
        );
    }

    if text(event, "change_class") == "payload_retrieval"
        && text(event, "source_authority_state") != "qualified"
    {
        return projection(
            claim_id,
            "insufficient_evidence",
            vec!["ISSUER_AUTHORITY_UNQUALIFIED".to_owned()],
        );
    }

    if text(graph, "coverage_state") != "complete"
        || !records(graph, "excluded_domains").is_empty()
        || text(report, "report_state") != "qualified"
        || text(report, "query_boundary") != "declared_fixture_graph"
        || text(report, "coverage_receipt_id").is_empty()
    {
        if finding_reasons.len() == 1
            && finding_reasons[0] == "MANDATORY_DEPENDENCY_EDGE_UNRESOLVED"
        {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["MANDATORY_DEPENDENCY_EDGE_UNRESOLVED".to_owned()],
            );
        }
        return projection(
            claim_id,
            "insufficient_evidence",
            vec!["GRAPH_COVERAGE_INCOMPLETE".to_owned()],
        );
    }

    let snapshot_digest = text(graph, "snapshot_digest");
    if !is_digest(snapshot_digest)
        || text(report, "graph_snapshot_digest") != snapshot_digest
        || text(report, "base_graph_snapshot_digest") != snapshot_digest
        || text(report, "event_id") != text(event, "event_id")
        || text(report, "scope_id") != text(query, "scope_id")
        || text(report, "scope_digest") != text(query, "scope_digest")
    {
        reasons.insert("IMPACT_REPORT_BINDING_MISMATCH".to_owned());
        return projection(claim_id, "invalid_evidence", reasons.into_iter().collect());
    }

    if text(graph, "integrity_state") == "cycle_detected" || graph_has_cycle(graph) {
        return projection(
            claim_id,
            "invalid_evidence",
            vec!["DEPENDENCY_GRAPH_CYCLE".to_owned()],
        );
    }

    if text(event, "change_class") == "impact_delta"
        && text(report, "delta_base_snapshot_digest") != snapshot_digest
    {
        return projection(
            claim_id,
            "invalid_evidence",
            vec!["IMPACT_DELTA_BASE_MISMATCH".to_owned()],
        );
    }

    if text(event, "change_class") == "invalidation_sequence" {
        let (Some(event_sequence), Some(last_sequence)) = (
            number(event, "owner_sequence"),
            number(graph, "last_owner_sequence"),
        ) else {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["EVENT_SEQUENCE_EVIDENCE_MISSING".to_owned()],
            );
        };
        if event_sequence <= last_sequence
            || number(report, "owner_sequence") != Some(event_sequence)
        {
            return projection(
                claim_id,
                "invalid_evidence",
                vec!["EVENT_SEQUENCE_DUPLICATE_OR_OUT_OF_ORDER".to_owned()],
            );
        }
    }

    if text(event, "change_class") == "calibration_expiry" {
        let Some(measurement_at) = number(claim, "measurement_at") else {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["CALIBRATION_MEASUREMENT_TIME_MISSING".to_owned()],
            );
        };
        let effective_at = number(event, "effective_at").unwrap_or(u64::MAX);
        if measurement_at < effective_at {
            return projection(
                claim_id,
                "not_affected",
                vec!["CALIBRATION_VALID_AT_MEASUREMENT".to_owned()],
            );
        }
        if measurement_at > number(query, "evaluation_at").unwrap_or(0) {
            return projection(
                claim_id,
                "revalidation_required",
                vec!["CALIBRATION_NOT_VALID_FOR_FUTURE_MEASUREMENT".to_owned()],
            );
        }
        return projection(
            claim_id,
            "invalid_evidence",
            vec!["CALIBRATION_INVALID_AT_MEASUREMENT".to_owned()],
        );
    }

    if text(event, "change_class") == "policy_update"
        && number(event, "effective_at").unwrap_or(0) > number(query, "evaluation_at").unwrap_or(0)
    {
        return projection(
            claim_id,
            "not_affected",
            vec!["NEW_EDITION_NOT_YET_EFFECTIVE".to_owned()],
        );
    }

    if text(claim, "evaluation_mode") == "historical"
        && number(query, "historical_evaluation_at").is_some_and(|historical| {
            number(event, "recorded_at").is_some_and(|recorded| recorded > historical)
        })
    {
        return projection(
            claim_id,
            "not_affected",
            vec!["NOT_KNOWN_AT_HISTORICAL_EVALUATION_TIME".to_owned()],
        );
    }

    let impacted = string_array(report, "impacted_claim_ids");
    let finding_disposition = text(finding, "disposition");
    let dependency_subjects = string_array(claim, "dependency_subjects");
    let direct_dependency = dependency_subjects.contains(text(event, "subject_node_id"));
    let has_path = contains_edge(graph, text(event, "subject_node_id"), claim_id);
    let finding_path = finding
        .get("dependency_path")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let path_is_exact = finding_path.first().and_then(Value::as_str)
        == Some(text(event, "subject_node_id"))
        && finding_path.last().and_then(Value::as_str) == Some(claim_id);

    if finding_disposition == "revalidate" || finding_disposition == "invalidate" {
        if !impacted.contains(claim_id) || !direct_dependency || !has_path || !path_is_exact {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["MANDATORY_DEPENDENCY_PATH_UNRESOLVED".to_owned()],
            );
        }

        if finding_disposition == "invalidate" {
            return projection(claim_id, "invalid_evidence", finding_reasons);
        }

        if let Some(receipt) = revalidation_for(input, claim_id) {
            if receipt_is_exact(receipt, claim, event, query, graph, report) {
                return projection(
                    claim_id,
                    "applicable_current",
                    vec!["EXACT_POST_CHANGE_REVALIDATION_ACCEPTED".to_owned()],
                );
            }
            if text(receipt, "subject_revision") != text(event, "source_revision_after") {
                return projection(
                    claim_id,
                    "revalidation_required",
                    vec!["REVALIDATION_REVISION_MISMATCH".to_owned()],
                );
            }
        }

        if !finding_reasons.is_empty() {
            return projection(claim_id, "revalidation_required", finding_reasons);
        }
        return projection(
            claim_id,
            "revalidation_required",
            vec!["POST_CHANGE_REVALIDATION_MISSING".to_owned()],
        );
    }

    if finding_disposition == "no_impact" {
        if impacted.contains(claim_id) {
            return projection(
                claim_id,
                "invalid_evidence",
                vec!["IMPACT_REPORT_FINDING_CONFLICT".to_owned()],
            );
        }
        if text(claim, "record_class") == "historical_record" {
            return projection(
                claim_id,
                "applicable_current",
                vec!["HISTORICAL_RECORD_IMMUTABLE".to_owned()],
            );
        }
        if finding
            .get("coverage_proof_id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["NO_IMPACT_COVERAGE_PROOF_MISSING".to_owned()],
            );
        }
        if direct_dependency && text(event, "change_class") != "calibration_expiry" {
            return projection(
                claim_id,
                "insufficient_evidence",
                vec!["OWNER_NO_IMPACT_CONTRADICTS_DECLARED_DEPENDENCY".to_owned()],
            );
        }
        return projection(claim_id, "not_affected", finding_reasons);
    }

    if finding_disposition == "insufficient" {
        return projection(claim_id, "insufficient_evidence", finding_reasons);
    }

    projection(
        claim_id,
        "insufficient_evidence",
        vec!["IMPACT_FINDING_DISPOSITION_UNSUPPORTED".to_owned()],
    )
}

/// Validate and project one synthetic change-impact case using only its input.
pub fn evaluate_case(case: &Value) -> Value {
    let input = case.get("input").unwrap_or(&Value::Null);
    let event = input.get("event").unwrap_or(&Value::Null);
    let query = input.get("query").unwrap_or(&Value::Null);
    let graph = input.get("graph").unwrap_or(&Value::Null);
    let report = graph.get("impact_report").unwrap_or(&Value::Null);

    let mut seen_claims = BTreeSet::new();
    let mut outputs = Vec::new();
    for claim in records(input, "claims") {
        let claim_id = text(claim, "claim_id");
        if claim_id.is_empty() || !seen_claims.insert(claim_id.to_owned()) {
            outputs.push(projection(
                claim_id,
                "invalid_evidence",
                vec!["DUPLICATE_OR_MISSING_CLAIM_ID".to_owned()],
            ));
            continue;
        }
        outputs.push(evaluate_claim(input, claim, event, query, graph, report));
    }
    json!({
        "id": text(case, "id"),
        "claim_applicability": outputs,
        "action_authority": "none",
        "claim_ceiling": CLAIM_CEILING
    })
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            actual.len() == expected.len()
                && actual.iter().all(|(key, value)| {
                    expected
                        .get(key)
                        .is_some_and(|expected_value| matches_expected(value, expected_value))
                })
        }
        (Value::Array(actual), Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(left, right)| matches_expected(left, right))
        }
        _ => actual == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn corpus() -> Value {
        let source = include_str!(
            "../../../docs/research/city-change-impact/CITY_CHANGE_IMPACT_CORPUS_V0_1.json"
        );
        serde_json::from_str(source).expect("change impact corpus parses")
    }

    #[test]
    fn all_frozen_change_impact_cases_match_known_answers() {
        let corpus = corpus();
        let cases = corpus["cases"].as_array().expect("cases array");
        assert_eq!(cases.len(), 28);
        for case in cases {
            let actual = evaluate_case(case);
            assert!(
                matches_expected(
                    &actual["claim_applicability"],
                    &case["expected"]["claim_applicability"]
                ),
                "{}: actual={} expected={}",
                text(case, "id"),
                actual["claim_applicability"],
                case["expected"]["claim_applicability"]
            );
        }
    }

    #[test]
    fn incomplete_coverage_never_produces_not_affected() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["graph"]["coverage_state"] = json!("incomplete");
        case["input"]["graph"]["excluded_domains"] = json!(["hidden-owner"]);
        let actual = evaluate_case(&case);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "insufficient_evidence"
        );
    }

    #[test]
    fn owner_report_scope_digest_must_match_the_query() {
        let mut case = corpus()["cases"][0].clone();
        case["input"]["graph"]["impact_report"]["scope_digest"] =
            json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        let actual = evaluate_case(&case);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "invalid_evidence"
        );
        assert_eq!(
            actual["claim_applicability"][0]["reason_codes"][0],
            "IMPACT_REPORT_BINDING_MISMATCH"
        );
    }

    #[test]
    fn untrusted_revocation_does_not_invalidate_a_claim() {
        let mut case = corpus()["cases"][4].clone();
        case["input"]["event"]["authority_state"] = json!("unverified");
        case["input"]["event"]["trusted_at"] = Value::Null;
        let actual = evaluate_case(&case);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "insufficient_evidence"
        );
    }

    #[test]
    fn exact_revalidation_discharges_only_the_claim_with_exact_receipt() {
        let mut case = corpus()["cases"][24].clone();
        let receipt = &mut case["input"]["revalidation_receipts"][0];
        receipt["subject_revision"] = json!("R2");
        let actual = evaluate_case(&case);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "applicable_current"
        );
        assert_eq!(
            actual["claim_applicability"][1]["state"],
            "revalidation_required"
        );
    }

    #[test]
    fn a_wrong_graph_base_cannot_be_accepted_as_an_impact_delta() {
        let actual = evaluate_case(&corpus()["cases"][22]);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "invalid_evidence"
        );
    }

    #[test]
    fn graph_cycles_are_invalid_evidence_not_unknown_no_impact() {
        let actual = evaluate_case(&corpus()["cases"][21]);
        assert_eq!(
            actual["claim_applicability"][0]["state"],
            "invalid_evidence"
        );
    }
}
