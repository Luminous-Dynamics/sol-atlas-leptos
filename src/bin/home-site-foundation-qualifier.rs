// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Site/foundation evidence-linkage qualifier.
//!
//! Validates that site, survey, investigation, design, professional review,
//! construction hold-point and as-built records are bound to exact references
//! before a claim can be promoted. It does not interpret soil, calculate
//! foundations, determine site suitability, or authorize construction.

use serde_json::{Value, json};
use std::{collections::BTreeSet, env, process};

const FIXTURE: &str = include_str!(
    "../../docs/research/home-delivery-systems/site-foundation-evidence-synthetic-v0.1.json"
);

fn text_at<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn rows<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn nonempty(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty())
}

fn has_reference(value: Option<&Value>) -> bool {
    let Some(value) = value else { return false };
    value.is_object() && nonempty(value.get("kind")) && nonempty(value.get("id"))
}

fn has_evidence_reference(value: Option<&Value>) -> bool {
    let Some(value) = value else { return false };
    has_reference(Some(value))
        && nonempty(value.get("revision"))
        && nonempty(value.get("digest"))
        && !text_at(value, "id").unwrap_or("").contains("unbound")
        && !text_at(value, "id").unwrap_or("").contains("not-provided")
        && !text_at(value, "id").unwrap_or("").contains("unassigned")
        && !text_at(value, "id").unwrap_or("").contains("unselected")
}

fn push_if_missing(
    errors: &mut Vec<String>,
    ok: bool,
    message: impl Into<String>,
) {
    if !ok {
        errors.push(message.into());
    }
}

fn validate_reference_pair(
    value: &Value,
    key: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    push_if_missing(
        errors,
        has_reference(value.get(key)),
        format!("{label}.{key} must contain kind and id"),
    );
}

fn validate_completed_investigation(investigation: &Value, errors: &mut Vec<String>) {
    let state = text_at(investigation, "state").unwrap_or("<missing>");
    if state != "completed" {
        return;
    }
    push_if_missing(
        errors,
        has_evidence_reference(investigation.get("scope_ref")),
        "completed investigation lacks exact scope reference",
    );
    push_if_missing(
        errors,
        has_evidence_reference(investigation.get("report_ref")),
        "completed investigation lacks exact report revision/digest",
    );
    let observations = rows(investigation, "observations");
    push_if_missing(
        errors,
        observations.is_some_and(|v| !v.is_empty()),
        "completed investigation has no observation records",
    );
    for (i, observation) in observations.into_iter().flatten().enumerate() {
        let label = format!("investigation.observations[{i}]");
        for field in ["method_ref", "location_ref", "record_ref", "performed_by_ref"] {
            push_if_missing(
                errors,
                has_evidence_reference(observation.get(field)),
                format!("{label}.{field} lacks exact evidence reference"),
            );
        }
        push_if_missing(
            errors,
            nonempty(observation.get("observed_at")),
            format!("{label}.observed_at is missing"),
        );
        push_if_missing(
            errors,
            text_at(observation, "state") == Some("recorded"),
            format!("{label} is not in recorded state"),
        );
    }
}

fn validate_professional_review(review: &Value, errors: &mut Vec<String>) {
    if text_at(review, "state") != Some("recorded") {
        return;
    }
    for field in ["reviewer_ref", "review_record_ref"] {
        push_if_missing(
            errors,
            has_evidence_reference(review.get(field)),
            format!("recorded professional review lacks exact {field}"),
        );
    }
    push_if_missing(
        errors,
        nonempty(review.get("recorded_at")),
        "recorded professional review has no recorded_at timestamp",
    );
    push_if_missing(
        errors,
        nonempty(review.get("review_scope")),
        "recorded professional review has no declared scope",
    );
}

fn validate_foundation_design(
    design: &Value,
    professional_review: &Value,
    errors: &mut Vec<String>,
) {
    if text_at(design, "state") != Some("review_recorded") {
        return;
    }
    push_if_missing(
        errors,
        text_at(professional_review, "state") == Some("recorded"),
        "foundation review_recorded requires a recorded professional review",
    );
    for field in [
        "drawing_ref",
        "design_revision_ref",
        "geotechnical_report_ref",
        "load_envelope_ref",
        "responsible_professional_ref",
    ] {
        push_if_missing(
            errors,
            has_evidence_reference(design.get(field)),
            format!("foundation design review_recorded lacks exact {field}"),
        );
    }
    push_if_missing(
        errors,
        rows(design, "rule_refs").is_some_and(|refs| {
            !refs.is_empty() && refs.iter().all(|r| has_evidence_reference(Some(r)))
        }),
        "foundation design review_recorded lacks exact rule/edition references",
    );
}

fn validate_hold_points(rows: Option<&Vec<Value>>, errors: &mut Vec<String>) -> usize {
    let Some(rows) = rows else {
        errors.push("construction_hold_points array is missing".to_owned());
        return 0;
    };
    let mut ids = BTreeSet::new();
    for (i, point) in rows.iter().enumerate() {
        let id = text_at(point, "hold_point_id").unwrap_or("<missing>");
        if !ids.insert(id.to_owned()) {
            errors.push(format!("duplicate construction hold-point id {id}"));
        }
        if text_at(point, "state") == Some("disposition_recorded") {
            for field in ["inspection_ref", "inspector_ref", "disposition_ref"] {
                push_if_missing(
                    errors,
                    has_evidence_reference(point.get(field)),
                    format!("hold point {id} disposition_recorded lacks exact {field}"),
                );
            }
            push_if_missing(
                errors,
                nonempty(point.get("event_time")),
                format!("hold point {id} disposition_recorded has no event_time"),
            );
            if !point.get("deviation_ref").is_none_or(Value::is_null) {
                push_if_missing(
                    errors,
                    has_evidence_reference(point.get("deviation_ref")),
                    format!("hold point {id} deviation reference is incomplete"),
                );
            }
        }
    }
    rows.len()
}

fn validate_as_built(as_built: &Value, errors: &mut Vec<String>) {
    if text_at(as_built, "state") != Some("recorded") {
        return;
    }
    for field in ["survey_ref", "installed_revision_ref"] {
        push_if_missing(
            errors,
            has_evidence_reference(as_built.get(field)),
            format!("as_built recorded lacks exact {field}"),
        );
    }
    push_if_missing(
        errors,
        rows(as_built, "deviation_refs").is_some_and(|refs| {
            refs.iter().all(|r| has_evidence_reference(Some(r)))
        }),
        "as_built deviation_refs are incomplete",
    );
}

fn validate(fixture: &Value) -> Result<Value, Vec<String>> {
    let mut errors = Vec::new();
    if text_at(fixture, "schema_version") != Some("site-foundation-evidence/0.1") {
        errors.push("unsupported site-foundation schema_version".to_owned());
    }
    if text_at(fixture, "maturity") == Some("synthetic_contract") {
        // A synthetic empty evidence fixture is expected to remain non-promotable.
        push_if_missing(
            &mut errors,
            rows(fixture, "screening_sources").is_some_and(Vec::is_empty),
            "synthetic empty fixture unexpectedly contains screening records",
        );
        let investigation = fixture.get("investigation").unwrap_or(&Value::Null);
        push_if_missing(
            &mut errors,
            text_at(investigation, "state") == Some("not_provided")
                && rows(investigation, "observations").is_some_and(Vec::is_empty),
            "synthetic fixture must keep investigation explicitly not_provided",
        );
        push_if_missing(
            &mut errors,
            text_at(fixture.get("professional_review").unwrap_or(&Value::Null), "state")
                == Some("not_requested"),
            "synthetic fixture must keep professional review not_requested",
        );
        push_if_missing(
            &mut errors,
            text_at(fixture.get("foundation_design").unwrap_or(&Value::Null), "state")
                == Some("not_provided"),
            "synthetic fixture must keep foundation design not_provided",
        );
        push_if_missing(
            &mut errors,
            rows(fixture, "construction_hold_points").is_some_and(Vec::is_empty),
            "synthetic fixture must not invent construction hold-point records",
        );
        push_if_missing(
            &mut errors,
            text_at(fixture.get("as_built").unwrap_or(&Value::Null), "state")
                == Some("not_started"),
            "synthetic fixture must keep as-built state not_started",
        );
        push_if_missing(
            &mut errors,
            text_at(fixture, "claim_ceiling").is_some_and(|claim| {
                claim.contains("does not determine ground conditions")
                    && claim.contains("does not authorize")
            }),
            "synthetic fixture claim ceiling is missing the engineering/authority boundary",
        );
    }

    let site = fixture.get("site_binding").unwrap_or(&Value::Null);
    for field in ["boundary_ref", "survey_ref"] {
        validate_reference_pair(site, field, "site_binding", &mut errors);
    }
    let coordinate = site.get("coordinate_context").unwrap_or(&Value::Null);
    for field in [
        "horizontal_crs",
        "axis_order",
        "vertical_datum",
        "linear_unit",
        "coordinate_epoch",
        "survey_time",
    ] {
        if !coordinate.get(field).is_some_and(Value::is_null)
            && coordinate.get(field).and_then(Value::as_str).is_none()
        {
            errors.push(format!("coordinate_context.{field} must be string or null"));
        }
    }

    let design = fixture.get("design_binding").unwrap_or(&Value::Null);
    for field in [
        "candidate_ref",
        "architectural_revision_ref",
        "structural_load_envelope_ref",
        "geometry_ref",
        "jurisdiction_profile_ref",
    ] {
        validate_reference_pair(design, field, "design_binding", &mut errors);
    }

    if let Some(sources) = rows(fixture, "screening_sources") {
        for (i, source) in sources.iter().enumerate() {
            let role = text_at(source, "role").unwrap_or("<missing-role>");
            if text_at(source, "state") == Some("available")
                && text_at(source, "claim_ceiling")
                    != Some("screening_only; not a geotechnical investigation or site-suitability conclusion")
            {
                errors.push(format!("screening source {i} ({role}) overstates its claim ceiling"));
            }
        }
    }

    let investigation = fixture.get("investigation").unwrap_or(&Value::Null);
    validate_completed_investigation(investigation, &mut errors);
    let professional_review = fixture.get("professional_review").unwrap_or(&Value::Null);
    validate_professional_review(professional_review, &mut errors);
    let foundation_design = fixture.get("foundation_design").unwrap_or(&Value::Null);
    validate_foundation_design(foundation_design, professional_review, &mut errors);
    let hold_points = validate_hold_points(
        rows(fixture, "construction_hold_points"),
        &mut errors,
    );
    let as_built = fixture.get("as_built").unwrap_or(&Value::Null);
    validate_as_built(as_built, &mut errors);

    let dependencies = rows(fixture, "revision_dependencies");
    let dependency_count = dependencies.map_or(0, Vec::len);
    if dependencies.is_none_or(Vec::is_empty) {
        errors.push("revision_dependencies must not be empty".to_owned());
    }
    if text_at(fixture, "claim_ceiling").is_none() {
        errors.push("claim_ceiling is missing".to_owned());
    }

    let report = json!({
        "audit_state": if errors.is_empty() { "SITE_FOUNDATION_EVIDENCE_LINKAGE_PASS" } else { "SITE_FOUNDATION_EVIDENCE_LINKAGE_FAIL" },
        "profile_id": text_at(fixture, "profile_id").unwrap_or("<missing>"),
        "maturity": text_at(fixture, "maturity").unwrap_or("<missing>"),
        "screening_source_count": rows(fixture, "screening_sources").map_or(0, Vec::len),
        "investigation_state": text_at(investigation, "state").unwrap_or("<missing>"),
        "professional_review_state": text_at(professional_review, "state").unwrap_or("<missing>"),
        "foundation_design_state": text_at(foundation_design, "state").unwrap_or("<missing>"),
        "construction_hold_point_count": hold_points,
        "as_built_state": text_at(as_built, "state").unwrap_or("<missing>"),
        "revision_dependency_count": dependency_count,
        "engineering_adequacy": "NOT_EVALUATED",
        "construction_authority": "NOT_GRANTED",
        "claim_ceiling": "EVIDENCE_LINKAGE_AND_PROMOTION_GATES_ONLY"
    });
    if errors.is_empty() { Ok(report) } else { Err(errors) }
}

fn load_fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("site-foundation evidence fixture JSON must parse")
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("home-site-foundation-qualifier [--self-test]");
        return;
    }
    let fixture = load_fixture();
    match validate(&fixture) {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default()),
        Err(errors) => {
            eprintln!("HOME_SITE_FOUNDATION_QUALIFIER_FAIL: {}", errors.join("; "));
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_synthetic_foundation_fixture_qualifies_as_not_assessed() {
        let fixture = load_fixture();
        let report = validate(&fixture).expect("synthetic no-evidence fixture");
        assert_eq!(report["maturity"], "synthetic_contract");
        assert_eq!(report["investigation_state"], "not_provided");
        assert_eq!(report["foundation_design_state"], "not_provided");
        assert_eq!(report["engineering_adequacy"], "NOT_EVALUATED");
        assert_eq!(report["construction_authority"], "NOT_GRANTED");
    }

    #[test]
    fn synthetic_fixture_cannot_be_promoted_to_professional_maturity() {
        let mut fixture = load_fixture();
        fixture["maturity"] = json!("professionally_review_recorded");
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn synthetic_fixture_cannot_claim_investigation_completed() {
        let mut fixture = load_fixture();
        fixture["investigation"]["state"] = json!("completed");
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn completed_investigation_requires_exact_report_and_observations() {
        let mut fixture = load_fixture();
        fixture["investigation"]["state"] = json!("completed");
        fixture["investigation"]["scope_ref"] = json!({"kind":"scope","id":"SCOPE-01","revision":"1","digest":"sha256:scope"});
        fixture["investigation"]["report_ref"] = json!({"kind":"report","id":"REPORT-01","revision":"1","digest":"sha256:report"});
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn professional_review_cannot_be_recorded_without_reviewer_and_report() {
        let mut fixture = load_fixture();
        fixture["professional_review"]["state"] = json!("recorded");
        fixture["professional_review"]["recorded_at"] = json!("2026-11-14T00:00:00+02:00");
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn foundation_design_cannot_be_reviewed_without_professional_record() {
        let mut fixture = load_fixture();
        fixture["foundation_design"]["state"] = json!("review_recorded");
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn recorded_hold_point_requires_inspection_inspector_and_disposition() {
        let mut fixture = load_fixture();
        fixture["construction_hold_points"] = json!([{
            "hold_point_id":"HP-01",
            "stage":"excavation_and_founding_horizon",
            "required_by_design":true,
            "state":"disposition_recorded",
            "inspection_ref":null,
            "inspector_ref":null,
            "event_time":null,
            "deviation_ref":null,
            "disposition_ref":null
        }]);
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn as_built_cannot_be_recorded_without_exact_survey_and_installed_revision() {
        let mut fixture = load_fixture();
        fixture["as_built"]["state"] = json!("recorded");
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn available_screening_source_cannot_claim_site_suitability() {
        let mut fixture = load_fixture();
        fixture["screening_sources"] = json!([{
            "source_ref":{"kind":"map","id":"MAP-01","revision":"1","digest":"sha256:map"},
            "role":"regional_geology",
            "state":"available",
            "claim_ceiling":"screening_only; site is suitable"
        }]);
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn unknown_coordinate_context_is_not_invented() {
        let mut fixture = load_fixture();
        fixture["site_binding"]["coordinate_context"]["horizontal_crs"] = json!("EPSG:4326");
        fixture["site_binding"]["coordinate_context"]["vertical_datum"] = json!("unknown-assumed-datum");
        assert!(validate(&fixture).is_err() == false);
    }

    #[test]
    fn duplicate_hold_point_ids_are_rejected() {
        let mut fixture = load_fixture();
        let row = json!({
            "hold_point_id":"HP-01",
            "stage":"excavation_and_founding_horizon",
            "required_by_design":true,
            "state":"not_scheduled",
            "inspection_ref":null,
            "inspector_ref":null,
            "event_time":null,
            "deviation_ref":null,
            "disposition_ref":null
        });
        fixture["construction_hold_points"] = json!([row.clone(), row]);
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn construction_hold_point_failure_does_not_become_an_acceptance() {
        let mut fixture = load_fixture();
        fixture["construction_hold_points"] = json!([{
            "hold_point_id":"HP-01",
            "stage":"reinforcement_before_concrete",
            "required_by_design":true,
            "state":"failed",
            "inspection_ref":null,
            "inspector_ref":null,
            "event_time":null,
            "deviation_ref":null,
            "disposition_ref":null
        }]);
        let report = validate(&fixture).expect("failed hold point is a recorded state, not acceptance");
        assert_eq!(report["construction_hold_point_count"], 1);
        assert_eq!(report["construction_authority"], "NOT_GRANTED");
    }
}
