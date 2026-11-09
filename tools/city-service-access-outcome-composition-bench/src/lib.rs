//! Independent, read-only composition reference for human-service outcomes.
//!
//! CityMorph owns network accessibility and capacity-potential calculations.
//! The restoration bridge owns work/continuity composition. City In-Use Outcomes
//! owns measurement qualification. This module only checks that their outputs
//! bind to one declared service, beneficiary frame, scope and time window.
//!
//! Potential access is never promoted to actual use. Missing consent is not a
//! negative experience result. This is a synthetic composition oracle, not a
//! service-delivery, clinical-quality, regulatory or professional authority.

use std::collections::BTreeSet;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const CORPUS_ID: &str = "CITY-SERVICE-ACCESS-OUTCOME-COMPOSITION-CORPUS-V0.1";
pub const CLAIM_CEILING: &str = "synthetic_cross_owner_composition_only";
pub const SCHEMA_ID: &str = "org.luminousdynamics.city-service-access-outcome-composition.corpus";

#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    pub output: Value,
    pub invalid: bool,
    pub not_comparable: bool,
    pub not_authorized: bool,
    pub insufficient: bool,
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn object<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite())
}

fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn add(reasons: &mut BTreeSet<String>, code: &str) {
    reasons.insert(code.to_owned());
}

fn same_window(left: &Value, right: &Value) -> bool {
    text(left, "window_id") == text(right, "window_id")
        && left.get("start_tick") == right.get("start_tick")
        && left.get("end_tick") == right.get("end_tick")
        && text(left, "tick_unit") == text(right, "tick_unit")
        && !text(left, "window_id").is_empty()
}

fn same_scope_fields(record: &Value, scope: &Value) -> bool {
    text(record, "common_scope_digest") == text(scope, "common_scope_digest")
        && text(record, "subject_id") == text(scope, "subject_id")
        && text(record, "subject_digest") == text(scope, "subject_digest")
        && text(record, "functional_unit") == text(scope, "functional_unit")
        && text(record, "group_id") == text(scope, "group_id")
        && text(record, "scenario_id") == text(scope, "scenario_id")
}

fn source_binding_matches(binding: &Value, source: &Value, scope: &Value) -> bool {
    if source.is_null() || !same_scope_fields(source, scope) {
        return false;
    }
    array(binding, "source_bindings")
        .iter()
        .filter(|candidate| {
            text(candidate, "owner") == text(source, "owner")
                && text(candidate, "contract_id") == text(source, "contract_id")
                && text(candidate, "contract_revision") == text(source, "contract_revision")
                && text(candidate, "receipt_id") == text(source, "receipt_id")
                && text(candidate, "source_scope_digest") == text(source, "source_scope_digest")
                && text(candidate, "subject_id") == text(scope, "subject_id")
                && text(candidate, "subject_digest") == text(scope, "subject_digest")
                && text(candidate, "functional_unit") == text(scope, "functional_unit")
                && text(candidate, "group_id") == text(scope, "group_id")
                && text(candidate, "scenario_id") == text(scope, "scenario_id")
                && text(candidate, "binding_state") == "Bound"
                && text(candidate, "evidence_state") == "current"
                && text(candidate, "rights_state") == "permitted"
                && text(candidate, "issuer_state") == "authorized"
                && text(candidate, "common_scope_digest") == text(binding, "common_scope_digest")
        })
        .count()
        == 1
}

fn add_source_binding_result(
    binding: &Value,
    source: &Value,
    scope: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
    code_prefix: &str,
) {
    if source.is_null() {
        add(reasons, &format!("{code_prefix}_SOURCE_MISSING"));
        *not_comparable = true;
        return;
    }
    let evidence_state = text(source, "evidence_state");
    let rights_state = text(source, "rights_state");
    let issuer_state = text(source, "issuer_state");
    if matches!(evidence_state, "revoked" | "disputed") {
        add(
            reasons,
            &format!("{code_prefix}_SOURCE_REVOKED_OR_DISPUTED"),
        );
        *invalid = true;
    } else if evidence_state != "current" {
        add(reasons, &format!("{code_prefix}_SOURCE_NOT_CURRENT"));
        *not_comparable = true;
    }
    if rights_state == "restricted" || rights_state == "revoked" {
        add(
            reasons,
            &format!("{code_prefix}_SOURCE_RIGHTS_NOT_PERMITTED"),
        );
        *not_authorized = true;
    } else if rights_state != "permitted" {
        add(reasons, &format!("{code_prefix}_SOURCE_RIGHTS_UNKNOWN"));
        *not_authorized = true;
    }
    if issuer_state != "authorized" {
        add(reasons, &format!("{code_prefix}_ISSUER_NOT_AUTHORIZED"));
        *not_authorized = true;
    }
    if !source_binding_matches(binding, source, scope) {
        add(
            reasons,
            &format!("{code_prefix}_SOURCE_NOT_BOUND_TO_EXACT_SCOPE"),
        );
        *not_comparable = true;
    }
    if !valid_sha256(text(source, "source_scope_digest"))
        || !valid_sha256(text(source, "subject_digest"))
    {
        add(reasons, &format!("{code_prefix}_SOURCE_DIGEST_INVALID"));
        *invalid = true;
    }
}

fn restoration_state(
    input: &Value,
    scope: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> String {
    let service = object(input, "service_contract");
    let restoration = object(input, "restoration");
    let reference = object(restoration, "source_ref");
    let mut local_invalid = false;
    let mut local_not_comparable = false;
    let mut local_not_authorized = false;

    add_source_binding_result(
        binding,
        reference,
        scope,
        reasons,
        &mut local_invalid,
        &mut local_not_comparable,
        &mut local_not_authorized,
        "RESTORATION",
    );
    *invalid |= local_invalid;
    *not_comparable |= local_not_comparable;
    *not_authorized |= local_not_authorized;

    if text(restoration, "service_id") != text(scope, "service_id")
        || text(restoration, "service_contract_revision") != text(service, "contract_revision")
        || text(restoration, "common_scope_digest") != text(scope, "common_scope_digest")
        || text(restoration, "functional_unit") != text(scope, "functional_unit")
        || text(restoration, "group_id") != text(scope, "group_id")
        || text(restoration, "scenario_id") != text(scope, "scenario_id")
    {
        add(reasons, "RESTORATION_SCOPE_OR_SERVICE_MISMATCH");
        local_not_comparable = true;
        *not_comparable = true;
    }
    if !valid_sha256(text(restoration, "output_sha256"))
        || !valid_sha256(text(restoration, "input_sha256"))
        || !valid_sha256(text(restoration, "receipt_sha256"))
        || !valid_commit(text(restoration, "source_commit"))
    {
        add(reasons, "RESTORATION_RECEIPT_BINDING_INVALID");
        local_invalid = true;
        *invalid = true;
    }

    let disposition = text(restoration, "bridge_disposition");
    let qualified = bool_value(restoration, "restoration_qualified_for_scope") == Some(true);
    let continuity = text(restoration, "continuity_state");
    let stable = object(restoration, "stable_adequacy");
    let stable_ok = bool_value(stable, "qualified") == Some(true)
        && stable
            .get("observed_ticks")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= stable
                .get("minimum_ticks")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX);
    let min_ticks = object(service, "stability_policy")
        .get("minimum_contiguous_adequate_ticks")
        .and_then(Value::as_u64);
    let receipt_ticks = stable.get("minimum_ticks").and_then(Value::as_u64);
    if min_ticks.is_none() || min_ticks != receipt_ticks {
        add(reasons, "RESTORATION_STABILITY_POLICY_MISMATCH");
        local_not_comparable = true;
        *not_comparable = true;
    }
    if local_invalid || local_not_comparable || local_not_authorized {
        return "not_qualified".to_owned();
    }
    if disposition == "RestoredForDeclaredScope"
        && qualified
        && continuity == "Adequate"
        && stable_ok
    {
        "restored".to_owned()
    } else if matches!(disposition, "Invalid" | "InvalidEvidence") {
        add(reasons, "RESTORATION_EVIDENCE_INVALID");
        *invalid = true;
        "invalid".to_owned()
    } else if matches!(disposition, "NotRestored" | "ProvisionalRecovery")
        || matches!(continuity, "Constrained" | "Inaccessible")
    {
        add(reasons, "RESTORATION_NOT_QUALIFIED_FOR_DECLARED_SCOPE");
        "not_restored".to_owned()
    } else {
        add(reasons, "RESTORATION_STATE_INSUFFICIENT");
        "unknown".to_owned()
    }
}

fn access_state(
    input: &Value,
    scope: &Value,
    frame: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> String {
    let access = object(input, "access");
    let source = object(access, "source_ref");
    add_source_binding_result(
        binding,
        source,
        scope,
        reasons,
        invalid,
        not_comparable,
        not_authorized,
        "ACCESS",
    );
    if !valid_sha256(text(access, "input_sha256"))
        || !valid_sha256(text(access, "output_sha256"))
        || !valid_sha256(text(access, "receipt_sha256"))
        || !valid_commit(text(access, "source_commit"))
    {
        add(reasons, "ACCESS_RECEIPT_BINDING_INVALID");
        *invalid = true;
    }
    if text(access, "case_id").is_empty()
        || text(access, "contract_id") != "CITY-MORPH-ACCESS-001"
        || text(access, "schema_version") != "0.1.0"
        || text(access, "scope_digest") != text(scope, "common_scope_digest")
        || text(access, "service_id") != text(scope, "service_id")
        || text(access, "group_id") != text(scope, "group_id")
        || text(access, "scenario_id") != text(scope, "scenario_id")
        || !same_window(
            object(access, "observation_window"),
            object(scope, "observation_window"),
        )
    {
        add(reasons, "ACCESS_SCOPE_OR_WINDOW_MISMATCH");
        *not_comparable = true;
    }

    let population = object(access, "population_group");
    let group_results = array(access, "group_results");
    if group_results.len() != 1 {
        if group_results.len() > 1 {
            add(reasons, "ACCESS_GROUP_RESULT_DUPLICATE");
            *invalid = true;
        } else {
            add(reasons, "ACCESS_GROUP_RESULT_MISSING");
            *not_comparable = true;
        }
        return "invalid".to_owned();
    }
    let result = &group_results[0];
    if text(population, "id") != text(scope, "group_id")
        || text(population, "frame_id") != text(frame, "frame_id")
        || text(result, "group_id") != text(scope, "group_id")
        || number(population, "demand") != number(frame, "demand")
        || text(population, "demand_unit") != text(frame, "demand_unit")
        || number(result, "demand") != number(frame, "demand")
        || text(result, "demand_unit") != text(frame, "demand_unit")
        || population.get("population") != frame.get("population_count")
    {
        add(reasons, "ACCESS_DEMAND_FRAME_MISMATCH");
        *not_comparable = true;
    }
    if !matches!(
        text(access, "disposition"),
        "FeasibleForSyntheticCase" | "Infeasible" | "Unknown" | "Unsupported"
    ) {
        add(reasons, "ACCESS_DISPOSITION_UNSUPPORTED");
        *invalid = true;
    }

    match text(result, "status") {
        "AccessibleAndAdequatePotential" => "accessible_and_capacity_adequate_potential".to_owned(),
        "AccessibleButCapacityConstrained" => {
            add(reasons, "POTENTIAL_ACCESS_CAPACITY_CONSTRAINED");
            "capacity_constrained_potential".to_owned()
        }
        "Inaccessible" => {
            add(reasons, "POTENTIAL_ACCESS_INFEASIBLE");
            "inaccessible".to_owned()
        }
        "Unknown" => {
            add(reasons, "POTENTIAL_ACCESS_UNKNOWN");
            "unknown".to_owned()
        }
        _ => {
            add(reasons, "ACCESS_GROUP_STATUS_UNSUPPORTED");
            *invalid = true;
            "invalid".to_owned()
        }
    }
}

fn frame_state(
    frame: &Value,
    scope: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> String {
    let source = object(frame, "source_ref");
    add_source_binding_result(
        binding,
        source,
        scope,
        reasons,
        invalid,
        not_comparable,
        not_authorized,
        "FRAME",
    );

    if text(frame, "frame_id").is_empty()
        || !valid_sha256(text(frame, "frame_sha256"))
        || text(frame, "group_id") != text(scope, "group_id")
        || text(frame, "functional_unit") != text(scope, "functional_unit")
        || text(frame, "scenario_id") != text(scope, "scenario_id")
    {
        add(reasons, "BENEFICIARY_FRAME_IDENTITY_MISMATCH");
        *not_comparable = true;
    }
    match text(frame, "demand_state") {
        "known_positive" => {
            if number(frame, "demand").is_none_or(|x| x <= 0.0)
                || text(frame, "demand_unit").is_empty()
            {
                add(reasons, "BENEFICIARY_DEMAND_VALUE_INVALID");
                *invalid = true;
            }
        }
        "verified_zero" => {
            if number(frame, "demand") != Some(0.0) || text(frame, "demand_unit").is_empty() {
                add(reasons, "VERIFIED_ZERO_DEMAND_CONTRADICTS_VALUE");
                *invalid = true;
            }
        }
        "unknown" | "not_collected" => {
            add(reasons, "BENEFICIARY_DEMAND_NOT_ESTABLISHED");
            return "not_established".to_owned();
        }
        _ => {
            add(reasons, "BENEFICIARY_DEMAND_STATE_UNSUPPORTED");
            *invalid = true;
            return "invalid".to_owned();
        }
    }
    if text(frame, "evidence_state") != "current" {
        add(reasons, "BENEFICIARY_FRAME_NOT_CURRENT");
        *not_comparable = true;
    }
    if text(frame, "frame_state") != "qualified" {
        add(reasons, "BENEFICIARY_FRAME_NOT_QUALIFIED");
        *not_comparable = true;
    }
    if frame
        .get("population_count")
        .and_then(Value::as_u64)
        .is_none_or(|v| v == 0)
    {
        add(reasons, "BENEFICIARY_POPULATION_COUNT_INVALID");
        *invalid = true;
    }
    if *invalid {
        "invalid".to_owned()
    } else if *not_authorized {
        "not_authorized".to_owned()
    } else if *not_comparable {
        "not_comparable".to_owned()
    } else {
        "qualified".to_owned()
    }
}

fn in_use_state(
    input: &Value,
    scope: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> String {
    let in_use = object(input, "in_use");
    let source = object(in_use, "source_ref");
    add_source_binding_result(
        binding,
        source,
        scope,
        reasons,
        invalid,
        not_comparable,
        not_authorized,
        "IN_USE",
    );
    if !valid_sha256(text(in_use, "input_sha256"))
        || !valid_sha256(text(in_use, "output_sha256"))
        || !valid_sha256(text(in_use, "receipt_sha256"))
        || !valid_commit(text(in_use, "source_commit"))
    {
        add(reasons, "IN_USE_RECEIPT_BINDING_INVALID");
        *invalid = true;
    }
    if text(in_use, "contract_id") != "CITY-IN-USE-OUTCOMES-001"
        || text(in_use, "schema_version") != "0.1.0"
        || text(in_use, "scope_digest") != text(scope, "common_scope_digest")
        || text(in_use, "service_id") != text(scope, "service_id")
        || text(in_use, "group_id") != text(scope, "group_id")
        || text(in_use, "scenario_id") != text(scope, "scenario_id")
        || text(in_use, "design_revision") != text(scope, "design_revision")
        || text(in_use, "bom_revision") != text(scope, "bom_revision")
        || !same_window(
            object(in_use, "observation_window"),
            object(scope, "observation_window"),
        )
    {
        add(reasons, "IN_USE_SCOPE_OR_WINDOW_MISMATCH");
        *not_comparable = true;
    }
    if text(in_use, "disposition") != "Evaluated" {
        add(reasons, "IN_USE_EVALUATION_NOT_QUALIFIED");
        *not_comparable = true;
    }

    let service = object(input, "service_contract");
    let metric_id = text(service, "required_operational_metric_id");
    let metric = array(in_use, "metric_results")
        .iter()
        .find(|row| text(row, "metric_id") == metric_id);
    let Some(metric) = metric else {
        add(reasons, "REQUIRED_OPERATIONAL_METRIC_MISSING");
        return "not_collected".to_owned();
    };
    let metric_binding = array(in_use, "metric_bindings")
        .iter()
        .find(|row| text(row, "metric_id") == metric_id);
    let Some(metric_binding) = metric_binding else {
        add(reasons, "REQUIRED_OPERATIONAL_METRIC_BINDING_MISSING");
        *not_comparable = true;
        return "not_comparable".to_owned();
    };
    add_source_binding_result(
        object(input, "scope_binding"),
        object(metric_binding, "source_ref"),
        scope,
        reasons,
        invalid,
        not_comparable,
        not_authorized,
        "OPERATIONAL_METRIC",
    );
    if text(metric_binding, "evidence_class") != "service"
        || text(metric_binding, "evidence_kind") != "service_receipt"
        || text(metric_binding, "unit") != text(service, "required_operational_metric_unit")
        || text(metric_binding, "calibration_state") != "current"
        || number(metric_binding, "coverage_fraction").is_none_or(|v| {
            v < number(service, "minimum_operational_coverage_fraction").unwrap_or(1.0)
        })
        || text(metric, "observation_state") != "Current"
        || text(metric, "threshold_state") != "Pass"
        || number(metric, "observed_value").is_none()
        || text(metric, "unit") != text(service, "required_operational_metric_unit")
    {
        add(reasons, "REQUIRED_OPERATIONAL_METRIC_NOT_QUALIFIED");
        return "insufficient".to_owned();
    }
    "qualified".to_owned()
}

fn realized_use_state(
    input: &Value,
    scope: &Value,
    frame: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> (String, Option<f64>) {
    let realized = object(input, "realized_use");
    match text(realized, "state") {
        "not_collected" => {
            if realized.get("observed_value").is_some_and(|v| !v.is_null()) {
                add(reasons, "REALIZED_USE_VALUE_PRESENT_WHILE_NOT_COLLECTED");
                *invalid = true;
                return ("invalid".to_owned(), None);
            }
            add(reasons, "REALIZED_SERVICE_USE_NOT_COLLECTED");
            return ("not_collected".to_owned(), None);
        }
        "unknown" => {
            add(reasons, "REALIZED_SERVICE_USE_UNKNOWN");
            return ("unknown".to_owned(), None);
        }
        "not_comparable" => {
            add(reasons, "REALIZED_SERVICE_USE_NOT_COMPARABLE");
            *not_comparable = true;
            return ("not_comparable".to_owned(), None);
        }
        "observed" => {}
        _ => {
            add(reasons, "REALIZED_SERVICE_USE_STATE_UNSUPPORTED");
            *invalid = true;
            return ("invalid".to_owned(), None);
        }
    }

    let source = object(realized, "source_ref");
    add_source_binding_result(
        binding,
        source,
        scope,
        reasons,
        invalid,
        not_comparable,
        not_authorized,
        "REALIZED_USE",
    );
    let service = object(input, "service_contract");
    let value = number(realized, "observed_value");
    if text(realized, "metric_id") != text(service, "required_use_metric_id")
        || text(realized, "unit") != text(service, "required_use_metric_unit")
        || text(realized, "evidence_kind") != "service_receipt"
        || text(realized, "evidence_state") != "current"
        || text(realized, "frame_id") != text(frame, "frame_id")
        || text(realized, "frame_sha256") != text(frame, "frame_sha256")
        || text(realized, "scope_digest") != text(scope, "common_scope_digest")
        || !same_window(
            object(realized, "observation_window"),
            object(scope, "observation_window"),
        )
        || number(realized, "coverage_fraction").is_none_or(|v| {
            v < number(service, "minimum_realized_use_coverage_fraction").unwrap_or(1.0)
        })
        || value.is_none_or(|v| v < 0.0)
    {
        add(reasons, "REALIZED_USE_BINDING_OR_MEASURE_INVALID");
        *not_comparable = true;
        return ("not_comparable".to_owned(), None);
    }

    match text(frame, "demand_state") {
        "known_positive" => {
            let demand = number(frame, "demand").unwrap_or(0.0);
            let Some(observed) = value else {
                add(reasons, "REALIZED_USE_VALUE_MISSING");
                *invalid = true;
                return ("invalid".to_owned(), None);
            };
            ("observed".to_owned(), Some(observed / demand))
        }
        "verified_zero" => {
            if value != Some(0.0) {
                add(reasons, "ZERO_DEMAND_CONFLICTS_WITH_POSITIVE_SERVICE_USE");
                *invalid = true;
                ("invalid".to_owned(), None)
            } else {
                ("zero_demand_not_applicable".to_owned(), None)
            }
        }
        _ => {
            add(reasons, "REALIZED_USE_REQUIRES_QUALIFIED_DEMAND_FRAME");
            *not_comparable = true;
            ("not_comparable".to_owned(), None)
        }
    }
}

fn resident_experience_state(
    input: &Value,
    scope: &Value,
    frame: &Value,
    binding: &Value,
    reasons: &mut BTreeSet<String>,
    invalid: &mut bool,
    not_comparable: &mut bool,
    not_authorized: &mut bool,
) -> String {
    let experience = object(input, "resident_experience");
    match text(experience, "state") {
        "not_requested" => "not_requested".to_owned(),
        "consent_denied" => "consent_denied".to_owned(),
        "not_collected" => "not_collected".to_owned(),
        "observed" => {
            let source = object(experience, "source_ref");
            add_source_binding_result(
                binding,
                source,
                scope,
                reasons,
                invalid,
                not_comparable,
                not_authorized,
                "EXPERIENCE",
            );
            let in_use = object(input, "in_use");
            let metric_id = text(experience, "metric_id");
            let metric_output = array(in_use, "metric_results")
                .iter()
                .find(|row| text(row, "metric_id") == metric_id);
            let metric_binding = array(in_use, "metric_bindings")
                .iter()
                .find(|row| text(row, "metric_id") == metric_id);
            if text(experience, "consent_state") != "authorized"
                || text(experience, "purpose") != "service-assurance"
                || text(experience, "evidence_kind") != "consented_survey"
                || text(experience, "frame_id") != text(frame, "frame_id")
                || text(experience, "frame_sha256") != text(frame, "frame_sha256")
                || text(experience, "scope_digest") != text(scope, "common_scope_digest")
                || !same_window(
                    object(experience, "observation_window"),
                    object(scope, "observation_window"),
                )
                || number(experience, "coverage_fraction").is_none_or(|v| {
                    v < number(
                        object(input, "service_contract"),
                        "minimum_experience_coverage_fraction",
                    )
                    .unwrap_or(1.0)
                })
                || metric_output.is_none_or(|row| text(row, "observation_state") != "Current")
                || metric_binding.is_none_or(|row| {
                    text(row, "evidence_class") != "experience"
                        || text(row, "evidence_kind") != "consented_survey"
                })
                || metric_id.is_empty()
                || number(experience, "observed_value").is_none()
            {
                add(reasons, "RESIDENT_EXPERIENCE_EVIDENCE_NOT_QUALIFIED");
                *not_comparable = true;
                "not_comparable".to_owned()
            } else {
                "observed".to_owned()
            }
        }
        _ => {
            add(reasons, "RESIDENT_EXPERIENCE_STATE_UNSUPPORTED");
            *invalid = true;
            "invalid".to_owned()
        }
    }
}

pub fn evaluate_case(case: &Value) -> Evaluation {
    let case_id = text(case, "case_id");
    let input = object(case, "input");
    let scope = object(input, "scope");
    let binding = object(input, "scope_binding");
    let frame = object(input, "beneficiary_frame");
    let mut reasons = BTreeSet::new();
    let mut invalid = false;
    let mut not_comparable = false;
    let mut not_authorized = false;
    let mut insufficient = false;

    if case_id.is_empty() || text(input, "case_id") != case_id {
        add(&mut reasons, "CASE_IDENTITY_MISMATCH");
        invalid = true;
    }
    if text(input, "classification") != "synthetic_fixture_only" {
        add(
            &mut reasons,
            "NON_SYNTHETIC_INPUT_NOT_SUPPORTED_BY_REFERENCE_ORACLE",
        );
        invalid = true;
    }
    if [
        "service_id",
        "service_contract_revision",
        "common_scope_digest",
        "subject_id",
        "functional_unit",
        "group_id",
        "scenario_id",
        "design_revision",
        "bom_revision",
    ]
    .iter()
    .any(|field| text(scope, field).is_empty())
        || !valid_sha256(text(scope, "common_scope_digest"))
        || !valid_sha256(text(scope, "subject_digest"))
    {
        add(&mut reasons, "COMMON_SCOPE_IDENTITY_INVALID");
        invalid = true;
    }
    if !same_scope_fields(binding, scope)
        || text(binding, "binding_state") != "Bound"
        || text(binding, "common_scope_digest") != text(scope, "common_scope_digest")
    {
        add(&mut reasons, "COMMON_SCOPE_BINDING_NOT_EXACT");
        not_comparable = true;
    }
    let window = object(scope, "observation_window");
    if text(window, "window_id").is_empty()
        || text(window, "tick_unit").is_empty()
        || window
            .get("start_tick")
            .and_then(Value::as_i64)
            .zip(window.get("end_tick").and_then(Value::as_i64))
            .is_none_or(|(a, b)| b <= a)
    {
        add(&mut reasons, "OBSERVATION_WINDOW_INVALID");
        invalid = true;
    }

    let service = object(input, "service_contract");
    if text(service, "service_id") != text(scope, "service_id")
        || text(service, "contract_revision") != text(scope, "service_contract_revision")
        || text(service, "common_scope_digest") != text(scope, "common_scope_digest")
        || text(service, "functional_unit") != text(scope, "functional_unit")
        || text(service, "group_id") != text(scope, "group_id")
        || text(service, "scenario_id") != text(scope, "scenario_id")
        || !matches!(
            text(service, "owner"),
            "symthaea" | "sol-atlas" | "mycelix" | "symtropy"
        )
    {
        add(&mut reasons, "SERVICE_CONTRACT_SCOPE_MISMATCH");
        not_comparable = true;
    }
    if !valid_sha256(text(service, "receipt_sha256")) {
        add(&mut reasons, "SERVICE_CONTRACT_RECEIPT_INVALID");
        invalid = true;
    }
    add_source_binding_result(
        binding,
        object(service, "source_ref"),
        scope,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
        "SERVICE_CONTRACT",
    );

    let frame = frame_state(
        frame,
        scope,
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );
    let restoration = restoration_state(
        input,
        scope,
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );
    let access = access_state(
        input,
        scope,
        object(input, "beneficiary_frame"),
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );
    let operational = in_use_state(
        input,
        scope,
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );
    let (realized_use, delivery_ratio) = realized_use_state(
        input,
        scope,
        object(input, "beneficiary_frame"),
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );
    let experience = resident_experience_state(
        input,
        scope,
        object(input, "beneficiary_frame"),
        binding,
        &mut reasons,
        &mut invalid,
        &mut not_comparable,
        &mut not_authorized,
    );

    let frame_state = frame;
    if bool_value(service, "realized_use_required") == Some(true)
        && !matches!(
            realized_use.as_str(),
            "observed" | "zero_demand_not_applicable"
        )
    {
        insufficient = true;
    }
    if bool_value(service, "operational_metric_required") == Some(true)
        && operational != "qualified"
    {
        insufficient = true;
    }
    if restoration != "restored"
        || access == "inaccessible"
        || access == "capacity_constrained_potential"
    {
        insufficient = true;
    }
    if frame_state != "qualified" {
        insufficient = true;
    }

    let composition_state = if invalid {
        "invalid_evidence"
    } else if not_authorized {
        "not_authorized"
    } else if not_comparable {
        "not_comparable"
    } else if restoration == "not_restored" || restoration == "not_qualified" {
        "restoration_not_qualified"
    } else if access == "inaccessible" {
        "potential_access_infeasible"
    } else if access == "capacity_constrained_potential" {
        "potential_access_constrained"
    } else if access == "unknown" {
        "potential_access_unknown"
    } else if restoration == "unknown"
        || frame_state != "qualified"
        || operational != "qualified"
        || insufficient
    {
        "insufficient_evidence"
    } else if realized_use == "observed" {
        "realized_service_outcome_established"
    } else if realized_use == "zero_demand_not_applicable" {
        "restored_potentially_accessible_zero_demand"
    } else {
        "restored_and_potentially_accessible"
    };

    let output = json!({
        "case_id": case_id,
        "composition_state": composition_state,
        "restoration_state": restoration,
        "potential_access_state": access,
        "beneficiary_frame_state": frame_state,
        "operational_evidence_state": operational,
        "realized_service_use_state": realized_use,
        "delivery_to_declared_demand_ratio": delivery_ratio,
        "resident_experience_state": experience,
        "reason_codes": reasons.iter().cloned().collect::<Vec<_>>(),
        "claim_ceiling": CLAIM_CEILING,
        "action_authority": "none"
    });
    Evaluation {
        output,
        invalid,
        not_comparable,
        not_authorized,
        insufficient,
    }
}

pub fn validate_corpus(corpus: &Value) -> Vec<String> {
    let mut errors = BTreeSet::new();
    if text(corpus, "schema_id") != SCHEMA_ID
        || text(corpus, "schema_version") != "0.1.0"
        || text(corpus, "corpus_id") != CORPUS_ID
        || text(corpus, "classification") != "synthetic_fixture_only"
        || text(corpus, "claim_ceiling") != CLAIM_CEILING
        || text(corpus, "policy_note").is_empty()
    {
        add(&mut errors, "CORPUS_IDENTITY_OR_POLICY_INVALID");
    }
    if array(corpus, "cases").len() != 30 {
        add(&mut errors, "CORPUS_CASE_COUNT_NOT_30");
    }
    let mut ids = BTreeSet::new();
    for case in array(corpus, "cases") {
        let id = text(case, "case_id");
        if id.is_empty() || !ids.insert(id.to_owned()) {
            add(&mut errors, "CASE_ID_MISSING_OR_DUPLICATE");
        }
        if !case.get("expected").is_some_and(Value::is_object) {
            add(&mut errors, "CASE_EXPECTED_OUTPUT_MISSING");
        }
    }
    errors.into_iter().collect()
}

pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    actual == expected
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected, validate_corpus};
    use serde_json::Value;

    const CORPUS: &str = include_str!(
        "../../../docs/research/city-service-access-outcome/CITY_SERVICE_ACCESS_OUTCOME_CORPUS_V0_1.json"
    );

    fn corpus() -> Value {
        serde_json::from_str(CORPUS).expect("service access/outcome corpus parses")
    }

    #[test]
    fn all_thirty_frozen_composition_cases_match() {
        let corpus = corpus();
        let errors = validate_corpus(&corpus);
        assert!(errors.is_empty(), "{errors:?}");
        for case in corpus["cases"].as_array().unwrap() {
            let result = evaluate_case(case);
            assert!(
                matches_expected(&result.output, &case["expected"]),
                "{} actual={} expected={}",
                case["case_id"],
                result.output,
                case["expected"]
            );
        }
    }

    #[test]
    fn absence_of_use_is_not_promoted_to_zero_use() {
        let case = corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["case_id"] == "SAO-001")
            .unwrap()
            .clone();
        let result = evaluate_case(&case);
        assert_eq!(result.output["realized_service_use_state"], "not_collected");
        assert_eq!(
            result.output["delivery_to_declared_demand_ratio"],
            Value::Null
        );
    }

    #[test]
    fn consent_denial_is_not_a_negative_experience_result() {
        let case = corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["case_id"] == "SAO-004")
            .unwrap()
            .clone();
        let result = evaluate_case(&case);
        assert_eq!(result.output["resident_experience_state"], "consent_denied");
    }

    #[test]
    fn potential_access_does_not_imply_realized_service() {
        let case = corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["case_id"] == "SAO-001")
            .unwrap()
            .clone();
        let result = evaluate_case(&case);
        assert_eq!(
            result.output["potential_access_state"],
            "accessible_and_capacity_adequate_potential"
        );
        assert_ne!(result.output["realized_service_use_state"], "observed");
        assert_eq!(
            result.output["composition_state"],
            "restored_and_potentially_accessible"
        );
    }

    #[test]
    fn zero_demand_is_not_divided_by_zero_or_silently_assumed() {
        let case = corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["case_id"] == "SAO-022")
            .unwrap()
            .clone();
        let result = evaluate_case(&case);
        assert_eq!(
            result.output["delivery_to_declared_demand_ratio"],
            Value::Null
        );
        assert_eq!(
            result.output["realized_service_use_state"],
            "zero_demand_not_applicable"
        );
    }
}
