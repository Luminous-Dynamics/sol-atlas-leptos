use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const REQUIRED_ROOT: &[&str] = &[
    "schema_version", "profile_id", "claim", "decision_tier", "operating_envelope",
    "risk_assessment", "evidence_vector", "artifact_bindings", "authority", "validity",
    "disposition", "claim_ceiling",
];

const DIMENSIONS: &[&str] = &[
    "software_conformance", "numerical_verification", "physical_validation",
    "data_integrity_rights", "uncertainty_credibility", "generalization", "robustness",
    "operational_assurance", "safety_security_authority", "prospective_outcomes",
];

const ACTIONS: &[&str] = &[
    "view", "annotate", "recommend", "rank_options", "prepare_plan",
    "schedule_within_envelope", "execute_external_action",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

fn issue(out: &mut Vec<ValidationIssue>, code: &str, path: impl Into<String>, message: impl Into<String>) {
    out.push(ValidationIssue { code: code.to_owned(), path: path.into(), message: message.into() });
}

fn object<'a>(value: &'a Value, path: &str, out: &mut Vec<ValidationIssue>) -> Option<&'a Map<String, Value>> {
    match value.as_object() {
        Some(map) => Some(map),
        None => {
            issue(out, "TYPE_OBJECT_REQUIRED", path, "Expected a JSON object.");
            None
        }
    }
}

fn string<'a>(value: &'a Value, key: &str, path: &str, out: &mut Vec<ValidationIssue>) -> Option<&'a str> {
    match value.get(key).and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => Some(s),
        _ => {
            issue(out, "REQUIRED_NONEMPTY_STRING", format!("{path}/{key}"), "Expected a non-empty string.");
            None
        }
    }
}

fn bool_value(value: &Value, key: &str, path: &str, out: &mut Vec<ValidationIssue>) -> Option<bool> {
    match value.get(key).and_then(Value::as_bool) {
        Some(v) => Some(v),
        None => {
            issue(out, "REQUIRED_BOOLEAN", format!("{path}/{key}"), "Expected a boolean.");
            None
        }
    }
}

fn array<'a>(value: &'a Value, key: &str, path: &str, out: &mut Vec<ValidationIssue>) -> Option<&'a Vec<Value>> {
    match value.get(key).and_then(Value::as_array) {
        Some(v) => Some(v),
        None => {
            issue(out, "REQUIRED_ARRAY", format!("{path}/{key}"), "Expected an array.");
            None
        }
    }
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit())
}

// The RFC deliberately requires UTC seconds precision so validity ordering is
// lexicographically comparable and reproducible across implementations.
fn is_canonical_utc(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 20
        && b[4] == b'-' && b[7] == b'-' && b[10] == b'T'
        && b[13] == b':' && b[16] == b':' && b[19] == b'Z'
        && b.iter().enumerate().all(|(i, c)| {
            [4, 7, 10, 13, 16, 19].contains(&i) || c.is_ascii_digit()
        })
}

fn validate_validity(profile: &Value, out: &mut Vec<ValidationIssue>) {
    let Some(validity) = profile.get("validity") else { return; };
    let start = string(validity, "valid_from", "/validity", out);
    let review = string(validity, "review_by", "/validity", out);
    if let Some(s) = start {
        if !is_canonical_utc(s) {
            issue(out, "NONCANONICAL_VALID_FROM", "/validity/valid_from", "Use canonical UTC YYYY-MM-DDTHH:MM:SSZ.");
        }
    }
    if let Some(r) = review {
        if !is_canonical_utc(r) {
            issue(out, "NONCANONICAL_REVIEW_BY", "/validity/review_by", "Use canonical UTC YYYY-MM-DDTHH:MM:SSZ.");
        }
    }
    if let (Some(s), Some(r)) = (start, review) {
        if is_canonical_utc(s) && is_canonical_utc(r) && r <= s {
            issue(out, "INVALID_VALIDITY_WINDOW", "/validity/review_by", "review_by must be later than valid_from.");
        }
    }
}

fn validate_artifacts<'a>(
    profile: &'a Value,
    out: &mut Vec<ValidationIssue>,
) -> (BTreeMap<String, &'a Value>, BTreeSet<String>) {
    let mut by_id = BTreeMap::new();
    let mut roles = BTreeSet::new();
    let Some(items) = array(profile, "artifact_bindings", "", out) else {
        return (by_id, roles);
    };
    for (i, artifact) in items.iter().enumerate() {
        let path = format!("/artifact_bindings/{i}");
        if object(artifact, &path, out).is_none() { continue; }
        let id = string(artifact, "artifact_id", &path, out);
        let role = string(artifact, "role", &path, out);
        let repo = string(artifact, "repository", &path, out);
        let commit = string(artifact, "commit_sha", &path, out);
        let blob = string(artifact, "git_blob_sha", &path, out);
        let file_path = string(artifact, "path", &path, out);
        let rights = string(artifact, "rights_state", &path, out);
        let _ = string(artifact, "evidence_class", &path, out);

        if let Some(id) = id {
            if by_id.insert(id.to_owned(), artifact).is_some() {
                issue(out, "DUPLICATE_ARTIFACT_ID", format!("{path}/artifact_id"), "Artifact IDs must be unique.");
            }
        }
        if let Some(role) = role { roles.insert(role.to_owned()); }
        if let Some(repo) = repo {
            if repo.split('/').count() != 2 {
                issue(out, "INVALID_REPOSITORY", format!("{path}/repository"), "Repository must use owner/name form.");
            }
        }
        if let Some(commit) = commit {
            if !is_hex(commit, 40) {
                issue(out, "INVALID_COMMIT_SHA", format!("{path}/commit_sha"), "Expected a 40-character Git commit SHA.");
            }
        }
        if let Some(blob) = blob {
            if !is_hex(blob, 40) {
                issue(out, "INVALID_GIT_BLOB_SHA", format!("{path}/git_blob_sha"), "Expected a 40-character Git blob SHA-1.");
            }
        }
        if let Some(content) = artifact.get("content_sha256").and_then(Value::as_str) {
            if !is_hex(content, 64) {
                issue(out, "INVALID_CONTENT_SHA256", format!("{path}/content_sha256"), "Expected a 64-character SHA-256 digest.");
            }
        }
        if let Some(path_value) = file_path {
            if path_value.starts_with('/') || path_value.split('/').any(|p| p == "..") {
                issue(out, "UNSAFE_ARTIFACT_PATH", format!("{path}/path"), "Use a repository-relative path without parent traversal.");
            }
        }
        if let Some(rights) = rights {
            if !["verified", "restricted", "unknown", "not_applicable"].contains(&rights) {
                issue(out, "INVALID_RIGHTS_STATE", format!("{path}/rights_state"), "Unrecognized rights state.");
            }
        }
    }
    (by_id, roles)
}

fn validate_evidence(
    profile: &Value,
    artifact_ids: &BTreeMap<String, &Value>,
    out: &mut Vec<ValidationIssue>,
) {
    let Some(vector) = profile.get("evidence_vector") else { return; };
    if object(vector, "/evidence_vector", out).is_none() { return; }
    let disposition = profile.get("disposition").and_then(Value::as_str).unwrap_or("");
    let tier = profile.get("decision_tier").and_then(Value::as_str).unwrap_or("");
    let qualified = disposition == "qualified_for_profile";
    let mut evidence_ids = BTreeSet::new();

    for dimension in DIMENSIONS {
        let path = format!("/evidence_vector/{dimension}");
        let Some(entry) = vector.get(*dimension) else {
            issue(out, "MISSING_EVIDENCE_DIMENSION", &path, "Every evidence dimension must be present.");
            continue;
        };
        if object(entry, &path, out).is_none() { continue; }
        let state = string(entry, "state", &path, out).unwrap_or("");
        let requirement = string(entry, "requirement", &path, out).unwrap_or("");
        let _ = string(entry, "rationale", &path, out);
        let evidence = array(entry, "evidence", &path, out).cloned().unwrap_or_default();

        if !["established", "not_established", "unknown", "conflicted", "not_applicable"].contains(&state) {
            issue(out, "INVALID_EVIDENCE_STATE", format!("{path}/state"), "Unrecognized evidence state.");
        }
        if !["mandatory", "optional"].contains(&requirement) {
            issue(out, "INVALID_EVIDENCE_REQUIREMENT", format!("{path}/requirement"), "Requirement must be mandatory or optional.");
        }
        if state == "not_applicable" {
            if requirement != "optional" {
                issue(out, "MANDATORY_DIMENSION_NOT_APPLICABLE", &path, "A mandatory dimension cannot be marked not_applicable.");
            }
            if string(entry, "reviewer_id", &path, out).is_none() || string(entry, "reviewed_at", &path, out).is_none() {
                issue(out, "UNREVIEWED_NOT_APPLICABLE", &path, "not_applicable requires a reviewer and review timestamp.");
            }
        }
        if state == "established" && evidence.is_empty() {
            issue(out, "ESTABLISHED_WITHOUT_EVIDENCE", &path, "An established dimension requires at least one evidence record.");
        }
        if qualified && requirement == "mandatory" && state != "established" {
            issue(out, "QUALIFIED_WITH_UNMET_REQUIREMENT", &path, "A qualified profile requires every mandatory dimension to be established.");
        }
        if tier == "D3_autonomous_control" && qualified && (requirement != "mandatory" || state != "established") {
            issue(out, "D3_EVIDENCE_VECTOR_INCOMPLETE", &path, "D3 qualification requires all ten dimensions mandatory and established.");
        }

        for (i, item) in evidence.iter().enumerate() {
            let ev_path = format!("{path}/evidence/{i}");
            if object(item, &ev_path, out).is_none() { continue; }
            let evidence_id = string(item, "evidence_id", &ev_path, out);
            let artifact_id = string(item, "artifact_id", &ev_path, out);
            let _ = string(item, "method", &ev_path, out);
            let _ = string(item, "scope", &ev_path, out);
            let _ = string(item, "result", &ev_path, out);
            let _ = string(item, "observed_at", &ev_path, out);
            if let Some(id) = evidence_id {
                if !evidence_ids.insert(id.to_owned()) {
                    issue(out, "DUPLICATE_EVIDENCE_ID", format!("{ev_path}/evidence_id"), "Evidence IDs must be unique across the full profile.");
                }
            }
            if let Some(id) = artifact_id {
                if !artifact_ids.contains_key(id) {
                    issue(out, "DANGLING_EVIDENCE_ARTIFACT", format!("{ev_path}/artifact_id"), "Evidence artifact_id must reference an artifact binding.");
                }
            }
            if state == "established" {
                let _ = string(item, "verifier_id", &ev_path, out);
            }
        }
    }
}

fn validate_authority(
    profile: &Value,
    artifact_ids: &BTreeMap<String, &Value>,
    roles: &BTreeSet<String>,
    out: &mut Vec<ValidationIssue>,
) {
    let Some(authority) = profile.get("authority") else { return; };
    if object(authority, "/authority", out).is_none() { return; }
    let tier = profile.get("decision_tier").and_then(Value::as_str).unwrap_or("");
    let disposition = profile.get("disposition").and_then(Value::as_str).unwrap_or("");
    let qualified = disposition == "qualified_for_profile";
    let permitted: Vec<&str> = authority.get("permitted_actions").and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    let actuation = bool_value(authority, "actuation_enabled", "/authority", out).unwrap_or(false);
    let human = bool_value(authority, "human_approval_required", "/authority", out).unwrap_or(false);

    for action in &permitted {
        if !ACTIONS.contains(action) {
            issue(out, "UNKNOWN_AUTHORITY_ACTION", "/authority/permitted_actions", format!("Unsupported action: {action}"));
        }
    }
    if !qualified {
        if actuation {
            issue(out, "UNQUALIFIED_ACTUATION", "/authority/actuation_enabled", "Actuation must remain disabled until qualification is established.");
        }
        if permitted.iter().any(|a| !["view", "annotate"].contains(a)) {
            issue(out, "UNQUALIFIED_ACTION_PERMISSION", "/authority/permitted_actions", "Unqualified profiles may only view or annotate; recommendations and side effects are not permitted.");
        }
    } else {
        match tier {
            "D0_descriptive" => {
                if actuation || permitted.iter().any(|a| !["view", "annotate"].contains(a)) {
                    issue(out, "D0_AUTHORITY_ESCALATION", "/authority", "D0 is descriptive only and permits view/annotate actions.");
                }
            }
            "D1_advisory" => {
                if actuation || !human || permitted.iter().any(|a| ["schedule_within_envelope", "execute_external_action"].contains(a)) {
                    issue(out, "D1_AUTHORITY_ESCALATION", "/authority", "D1 is advisory, requires human approval and cannot schedule or actuate.");
                }
            }
            "D2_delegated_planning" => {
                if actuation || !human || !permitted.contains(&"schedule_within_envelope") || permitted.contains(&"execute_external_action") {
                    issue(out, "D2_AUTHORITY_INVALID", "/authority", "D2 requires human approval, may schedule within the envelope, and cannot actuate.");
                }
                require_role("independent_verifier", roles, out);
                require_role("rollback_plan", roles, out);
                require_authority_artifact(authority, "independent_verifier_artifact_id", "independent_verifier", artifact_ids, out);
                require_authority_artifact(authority, "rollback_plan_artifact_id", "rollback_plan", artifact_ids, out);
            }
            "D3_autonomous_control" => {
                if !actuation || !permitted.contains(&"execute_external_action") {
                    issue(out, "D3_AUTHORITY_INCOMPLETE", "/authority", "Qualified D3 requires actuation_enabled and execute_external_action.");
                }
                for role in ["independent_verifier", "rollback_plan", "safety_case", "operational_authorization", "cybersecurity_assessment"] {
                    require_role(role, roles, out);
                }
                require_authority_artifact(authority, "independent_verifier_artifact_id", "independent_verifier", artifact_ids, out);
                require_authority_artifact(authority, "rollback_plan_artifact_id", "rollback_plan", artifact_ids, out);
                require_authority_artifact(authority, "safety_case_artifact_id", "safety_case", artifact_ids, out);
                require_authority_artifact(authority, "operational_authorization_artifact_id", "operational_authorization", artifact_ids, out);
                require_authority_artifact(authority, "cybersecurity_assessment_artifact_id", "cybersecurity_assessment", artifact_ids, out);
            }
            _ => issue(out, "UNKNOWN_DECISION_TIER", "/decision_tier", "Unknown decision tier for authority evaluation."),
        }
    }
}

fn require_role(role: &str, roles: &BTreeSet<String>, out: &mut Vec<ValidationIssue>) {
    if !roles.contains(role) {
        issue(out, "MISSING_REQUIRED_ARTIFACT_ROLE", "/artifact_bindings", format!("Required artifact role is missing: {role}."));
    }
}

fn require_authority_artifact(
    authority: &Value,
    field: &str,
    expected_role: &str,
    artifact_ids: &BTreeMap<String, &Value>,
    out: &mut Vec<ValidationIssue>,
) {
    let Some(id) = string(authority, field, "/authority", out) else { return; };
    let Some(artifact) = artifact_ids.get(id) else {
        issue(out, "AUTHORITY_ARTIFACT_NOT_BOUND", format!("/authority/{field}"), "Authority artifact ID is not present in artifact_bindings.");
        return;
    };
    if artifact.get("role").and_then(Value::as_str) != Some(expected_role) {
        issue(out, "AUTHORITY_ARTIFACT_ROLE_MISMATCH", format!("/authority/{field}"), format!("Artifact must have role {expected_role}."));
    }
}

pub fn validate_profile(profile: &Value) -> Vec<ValidationIssue> {
    let mut out = Vec::new();
    let Some(root) = object(profile, "", &mut out) else { return out; };

    for key in REQUIRED_ROOT {
        if !root.contains_key(*key) {
            issue(&mut out, "MISSING_ROOT_FIELD", format!("/{key}"), "Required root field is missing.");
        }
    }
    if profile.get("schema_version").and_then(Value::as_str) != Some("luminous.industry-assurance-profile.v0.1") {
        issue(&mut out, "UNSUPPORTED_SCHEMA_VERSION", "/schema_version", "Unsupported IEAC profile schema version.");
    }
    let _ = string(profile, "profile_id", "", &mut out);
    let _ = string(profile, "claim_ceiling", "", &mut out);

    if let Some(claim) = profile.get("claim") {
        if object(claim, "/claim", &mut out).is_some() {
            for key in ["claim_id", "capability_id", "instance_id", "decision_id", "description", "intended_user"] {
                let _ = string(claim, key, "/claim", &mut out);
            }
        }
    }
    let tier = profile.get("decision_tier").and_then(Value::as_str).unwrap_or("");
    if !["D0_descriptive", "D1_advisory", "D2_delegated_planning", "D3_autonomous_control"].contains(&tier) {
        issue(&mut out, "INVALID_DECISION_TIER", "/decision_tier", "Unsupported decision tier.");
    }
    let disposition = profile.get("disposition").and_then(Value::as_str).unwrap_or("");
    if !["qualified_for_profile", "not_qualified", "indeterminate", "revoked"].contains(&disposition) {
        issue(&mut out, "INVALID_DISPOSITION", "/disposition", "Unsupported qualification disposition.");
    }

    if let Some(risk) = profile.get("risk_assessment") {
        if object(risk, "/risk_assessment", &mut out).is_some() {
            let class = string(risk, "reviewed_class", "/risk_assessment", &mut out).unwrap_or("");
            if !["low", "moderate", "high", "critical", "unassessed"].contains(&class) {
                issue(&mut out, "INVALID_RISK_CLASS", "/risk_assessment/reviewed_class", "Unsupported reviewed risk class.");
            }
            for key in ["rationale", "reviewer_id", "reviewed_at", "rule_set_version"] {
                let _ = string(risk, key, "/risk_assessment", &mut out);
            }
            let mut risk_dims = BTreeMap::new();
            if let Some(dims) = risk.get("dimensions") {
                if object(dims, "/risk_assessment/dimensions", &mut out).is_some() {
                    for key in ["consequence", "model_influence", "exposure", "reversibility", "detectability", "fallback"] {
                        let value = string(dims, key, "/risk_assessment/dimensions", &mut out).unwrap_or("");
                        risk_dims.insert(key, value);
                    }
                    let enums: [(&str, &[&str]); 6] = [
                        ("consequence", &["negligible", "minor", "major", "severe", "catastrophic", "unknown"]),
                        ("model_influence", &["informational", "advisory", "constraint_gating", "selection", "direct_actuation", "unknown"]),
                        ("exposure", &["single_asset", "site_or_facility", "multi_site", "regional", "national_or_larger", "unknown"]),
                        ("reversibility", &["readily_reversible", "costly_or_delayed", "difficult_or_irreversible", "unknown"]),
                        ("detectability", &["immediate_and_reliable", "delayed_or_partial", "poor", "unknown"]),
                        ("fallback", &["independent_tested_fallback", "human_fallback", "degraded_fallback", "no_verified_fallback", "unknown"])
                    ];
                    for (key, allowed) in enums {
                        if let Some(value) = risk_dims.get(key) {
                            if !allowed.contains(value) {
                                issue(&mut out, "INVALID_RISK_DIMENSION", format!("/risk_assessment/dimensions/{key}"), "Unrecognized risk dimension value.");
                            }
                        }
                    }
                }
            } else {
                issue(&mut out, "MISSING_RISK_DIMENSIONS", "/risk_assessment/dimensions", "Risk dimensions are required.");
            }
            if disposition == "qualified_for_profile" {
                if class == "unassessed" {
                    issue(&mut out, "QUALIFIED_WITH_UNASSESSED_RISK", "/risk_assessment/reviewed_class", "A qualified profile requires a reviewed risk class.");
                }
                if risk_dims.values().any(|value| *value == "unknown") {
                    issue(&mut out, "QUALIFIED_WITH_UNKNOWN_RISK", "/risk_assessment/dimensions", "A qualified profile cannot retain unknown risk dimensions.");
                }
                match tier {
                    "D2_delegated_planning" => {
                        if !["constraint_gating", "selection"].contains(&risk_dims.get("model_influence").copied().unwrap_or("")) {
                            issue(&mut out, "D2_RISK_INFLUENCE_MISMATCH", "/risk_assessment/dimensions/model_influence", "D2 requires a reviewed constraint-gating or selection influence.");
                        }
                        if !["independent_tested_fallback", "human_fallback"].contains(&risk_dims.get("fallback").copied().unwrap_or("")) {
                            issue(&mut out, "D2_FALLBACK_INSUFFICIENT", "/risk_assessment/dimensions/fallback", "D2 requires a human or independently tested fallback.");
                        }
                    }
                    "D3_autonomous_control" => {
                        if risk_dims.get("model_influence").copied() != Some("direct_actuation") {
                            issue(&mut out, "D3_RISK_INFLUENCE_MISMATCH", "/risk_assessment/dimensions/model_influence", "D3 requires direct_actuation to be declared.");
                        }
                        if risk_dims.get("fallback").copied() != Some("independent_tested_fallback") {
                            issue(&mut out, "D3_FALLBACK_INSUFFICIENT", "/risk_assessment/dimensions/fallback", "D3 requires an independently tested fallback.");
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let (artifact_ids, roles) = validate_artifacts(profile, &mut out);
    validate_evidence(profile, &artifact_ids, &mut out);
    validate_authority(profile, &artifact_ids, &roles, &mut out);
    validate_validity(profile, &mut out);

    if let Some(envelope) = profile.get("operating_envelope") {
        if object(envelope, "/operating_envelope", &mut out).is_some() {
            for key in ["envelope_id", "domain_description", "out_of_domain_policy"] {
                let _ = string(envelope, key, "/operating_envelope", &mut out);
            }
            if let Some(conditions) = array(envelope, "conditions", "/operating_envelope", &mut out) {
                for (i, condition) in conditions.iter().enumerate() {
                    let path = format!("/operating_envelope/conditions/{i}");
                    if object(condition, &path, &mut out).is_some() {
                        for key in ["variable", "unit", "support", "evidence_state"] {
                            let _ = string(condition, key, &path, &mut out);
                        }
                        if let Some(source_ids) = condition.get("source_artifact_ids").and_then(Value::as_array) {
                            for (j, source) in source_ids.iter().enumerate() {
                                if let Some(id) = source.as_str() {
                                    if !artifact_ids.contains_key(id) {
                                        issue(&mut out, "DANGLING_ENVELOPE_ARTIFACT", format!("{path}/source_artifact_ids/{j}"), "Source artifact ID is not bound.");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    out.sort_by(|a, b| (&a.path, &a.code, &a.message).cmp(&(&b.path, &b.code, &b.message)));
    out
}

pub fn build_report(profile: &Value, issues: &[ValidationIssue]) -> Value {
    let profile_id = profile.get("profile_id").and_then(Value::as_str).unwrap_or("unknown");
    let disposition = profile.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
    let valid = issues.is_empty();
    let status = if !valid {
        "PROFILE_CONTRACT_INVALID"
    } else if disposition == "qualified_for_profile" {
        "PROFILE_CONTRACT_VALID_QUALIFICATION_NOT_INDEPENDENTLY_ATTESTED"
    } else {
        "PROFILE_CONTRACT_VALID_NOT_QUALIFIED"
    };
    let issue_values: Vec<Value> = issues.iter().map(|i| serde_json::json!({
        "code": i.code, "path": i.path, "message": i.message
    })).collect();
    serde_json::json!({
        "schema_version": "luminous.industry-assurance-validation-report.v0.1",
        "profile_id": profile_id,
        "declared_disposition": disposition,
        "contract_valid": valid,
        "status": status,
        "independent_qualification_attested": false,
        "claim_ceiling": "Checks local structural and semantic consistency only. Does not fetch artifact bytes, establish source truth, rights, scientific validity, evidence independence, operational authorization, safety or regulatory compliance.",
        "issue_count": issues.len(),
        "issues": issue_values
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../../schemas/examples/industry-assurance-qualification-profile-synthetic-v0.1.json")).unwrap()
    }

    fn artifact(id: &str, role: &str) -> Value {
        json!({
            "artifact_id": id, "role": role, "repository": "Luminous-Dynamics/sol-atlas-leptos",
            "commit_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "path": "docs/example.md",
            "git_blob_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "content_sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "evidence_class": "synthetic", "rights_state": "verified"
        })
    }

    fn make_qualified_d1() -> Value {
        let mut p = fixture();
        p["disposition"] = json!("qualified_for_profile");
        p["authority"]["permitted_actions"] = json!(["view", "annotate", "recommend", "rank_options"]);
        p["authority"]["human_approval_required"] = json!(true);
        p["authority"]["actuation_enabled"] = json!(false);
        p["artifact_bindings"] = json!([artifact("evidence-001", "test_report")]);
        for dim in DIMENSIONS {
            p["evidence_vector"][*dim]["state"] = json!("established");
            p["evidence_vector"][*dim]["requirement"] = json!("mandatory");
            p["evidence_vector"][*dim]["evidence"] = json!([{
                "evidence_id": format!("ev-{dim}"),
                "artifact_id": "evidence-001",
                "method": "synthetic fixture check",
                "scope": "one synthetic fixture only",
                "result": "matched expected value",
                "observed_at": "2026-10-30T00:00:00Z",
                "verifier_id": "independent-fixture-checker"
            }]);
        }
        p
    }

    fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
        issues.iter().any(|i| i.code == code)
    }

    #[test]
    fn synthetic_example_is_valid_but_not_qualified() {
        let p = fixture();
        let issues = validate_profile(&p);
        assert!(issues.is_empty(), "{issues:#?}");
        let report = build_report(&p, &issues);
        assert_eq!(report["status"], "PROFILE_CONTRACT_VALID_NOT_QUALIFIED");
        assert!(!report["independent_qualification_attested"].as_bool().unwrap());
    }

    #[test]
    fn qualified_advisory_profile_passes_internal_consistency() {
        let p = make_qualified_d1();
        let issues = validate_profile(&p);
        assert!(issues.is_empty(), "{issues:#?}");
        let report = build_report(&p, &issues);
        assert_eq!(report["status"], "PROFILE_CONTRACT_VALID_QUALIFICATION_NOT_INDEPENDENTLY_ATTESTED");
        assert_eq!(report["independent_qualification_attested"], false);
    }

    #[test]
    fn unqualified_profile_cannot_recommend_or_actuate() {
        let mut p = fixture();
        p["authority"]["permitted_actions"] = json!(["view", "recommend"]);
        p["authority"]["actuation_enabled"] = json!(true);
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "UNQUALIFIED_ACTION_PERMISSION"));
        assert!(has_code(&issues, "UNQUALIFIED_ACTUATION"));
    }

    #[test]
    fn advisory_profile_cannot_schedule_or_actuate() {
        let mut p = make_qualified_d1();
        p["authority"]["permitted_actions"] = json!(["view", "schedule_within_envelope"]);
        p["authority"]["actuation_enabled"] = json!(true);
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "D1_AUTHORITY_ESCALATION"));
    }

    #[test]
    fn qualified_profile_cannot_hide_unestablished_mandatory_dimension() {
        let mut p = make_qualified_d1();
        p["evidence_vector"]["robustness"]["state"] = json!("unknown");
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "QUALIFIED_WITH_UNMET_REQUIREMENT"));
    }

    #[test]
    fn dangling_evidence_artifact_fails_closed() {
        let mut p = make_qualified_d1();
        p["evidence_vector"]["robustness"]["evidence"][0]["artifact_id"] = json!("not-bound");
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "DANGLING_EVIDENCE_ARTIFACT"));
    }

    #[test]
    fn duplicate_evidence_ids_fail_closed() {
        let mut p = make_qualified_d1();
        let duplicate = p["evidence_vector"]["software_conformance"]["evidence"][0]["evidence_id"].clone();
        p["evidence_vector"]["numerical_verification"]["evidence"][0]["evidence_id"] = duplicate;
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "DUPLICATE_EVIDENCE_ID"));
    }

    #[test]
    fn not_applicable_requires_optional_dimension_and_reviewer() {
        let mut p = fixture();
        p["evidence_vector"]["physical_validation"]["state"] = json!("not_applicable");
        p["evidence_vector"]["physical_validation"]["requirement"] = json!("mandatory");
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "MANDATORY_DIMENSION_NOT_APPLICABLE"));
        assert!(has_code(&issues, "UNREVIEWED_NOT_APPLICABLE"));
    }

    #[test]
    fn validity_window_must_be_ordered_and_canonical() {
        let mut p = fixture();
        p["validity"]["review_by"] = json!("2026-10-01T00:00:00Z");
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "INVALID_VALIDITY_WINDOW"));
    }

    #[test]
    fn d2_requires_independent_verifier_and_rollback_artifacts() {
        let mut p = make_qualified_d1();
        p["decision_tier"] = json!("D2_delegated_planning");
        p["risk_assessment"]["dimensions"]["model_influence"] = json!("selection");
        p["authority"]["permitted_actions"] = json!(["view", "annotate", "schedule_within_envelope"]);
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "MISSING_REQUIRED_ARTIFACT_ROLE"));
    }

    #[test]
    fn d3_requires_safety_and_authorization_artifacts() {
        let mut p = make_qualified_d1();
        p["decision_tier"] = json!("D3_autonomous_control");
        p["risk_assessment"]["dimensions"]["model_influence"] = json!("direct_actuation");
        p["risk_assessment"]["dimensions"]["fallback"] = json!("independent_tested_fallback");
        p["authority"]["actuation_enabled"] = json!(true);
        p["authority"]["permitted_actions"] = json!(["view", "execute_external_action"]);
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "MISSING_REQUIRED_ARTIFACT_ROLE"));
        assert!(!has_code(&issues, "D3_EVIDENCE_VECTOR_INCOMPLETE"));
    }

    #[test]
    fn sha1_git_object_and_sha256_content_fields_are_not_interchangeable() {
        let mut p = make_qualified_d1();
        p["artifact_bindings"][0]["git_blob_sha"] = json!("not-a-sha256");
        let issues = validate_profile(&p);
        assert!(has_code(&issues, "INVALID_GIT_BLOB_SHA"));
    }

    #[test]
    fn complete_d2_authority_bundle_passes_consistency() {
        let mut p = make_qualified_d1();
        p["decision_tier"] = json!("D2_delegated_planning");
        p["authority"]["permitted_actions"] = json!(["view", "annotate", "schedule_within_envelope"]);
        p["artifact_bindings"].as_array_mut().unwrap().push(artifact("verifier-001", "independent_verifier"));
        p["artifact_bindings"].as_array_mut().unwrap().push(artifact("rollback-001", "rollback_plan"));
        p["authority"]["independent_verifier_artifact_id"] = json!("verifier-001");
        p["authority"]["rollback_plan_artifact_id"] = json!("rollback-001");
        let issues = validate_profile(&p);
        assert!(issues.is_empty(), "{issues:#?}");
    }

    #[test]
    fn complete_d3_authority_bundle_passes_consistency() {
        let mut p = make_qualified_d1();
        p["decision_tier"] = json!("D3_autonomous_control");
        p["authority"]["actuation_enabled"] = json!(true);
        p["authority"]["permitted_actions"] = json!(["view", "execute_external_action"]);
        let roles = [
            ("verifier-001", "independent_verifier"),
            ("rollback-001", "rollback_plan"),
            ("safety-001", "safety_case"),
            ("authorization-001", "operational_authorization"),
            ("cyber-001", "cybersecurity_assessment")
        ];
        for (id, role) in roles {
            p["artifact_bindings"].as_array_mut().unwrap().push(artifact(id, role));
        }
        p["authority"]["independent_verifier_artifact_id"] = json!("verifier-001");
        p["authority"]["rollback_plan_artifact_id"] = json!("rollback-001");
        p["authority"]["safety_case_artifact_id"] = json!("safety-001");
        p["authority"]["operational_authorization_artifact_id"] = json!("authorization-001");
        p["authority"]["cybersecurity_assessment_artifact_id"] = json!("cyber-001");
        let issues = validate_profile(&p);
        assert!(issues.is_empty(), "{issues:#?}");
    }

    #[test]
    fn unqualified_d3_profile_remains_read_only() {
        let mut p = fixture();
        p["decision_tier"] = json!("D3_autonomous_control");
        let issues = validate_profile(&p);
        assert!(issues.is_empty(), "{issues:#?}");
    }


    #[test]
    fn profile_and_report_schemas_parse_with_expected_contract_versions() {
        let schema: Value = serde_json::from_str(include_str!("../../../schemas/industry-assurance-qualification-profile-v0.1.schema.json")).unwrap();
        let report_schema: Value = serde_json::from_str(include_str!("../../../schemas/industry-assurance-validation-report-v0.1.schema.json")).unwrap();
        assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
        assert_eq!(schema["properties"]["schema_version"]["const"], "luminous.industry-assurance-profile.v0.1");
        assert_eq!(report_schema["properties"]["independent_qualification_attested"]["const"], false);
        assert!(schema["properties"]["evidence_vector"]["required"].as_array().unwrap().len() == 10);
    }

}
