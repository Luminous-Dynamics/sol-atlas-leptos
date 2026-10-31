use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub const VALIDATOR_VERSION: &str = "industry-twin-interface-validator/0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: String,
    pub path: String,
}

impl ValidationIssue {
    fn new(code: &str, path: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            path: path.into(),
        }
    }
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn array_at<'a>(value: &'a Value, key: &str) -> Option<&'a [Value]> {
    value.get(key)?.as_array().map(Vec::as_slice)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}


fn reject_unknown_keys(
    value: &Value,
    allowed: &[&str],
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            issues.push(ValidationIssue::new(
                "UNKNOWN_PROPERTY_NOT_ALLOWED",
                format!("{path}.{key}"),
            ));
        }
    }
}

fn valid_scope_list(value: &Value, field: &str, suffix: &str, issues: &mut Vec<ValidationIssue>) {
    let Some(scopes) = array_at(value, field) else {
        issues.push(ValidationIssue::new(
            "AUTHORIZATION_SCOPE_LIST_INVALID",
            format!("authorization.{field}"),
        ));
        return;
    };
    let mut seen = HashSet::new();
    for (index, scope) in scopes.iter().enumerate() {
        let path = format!("authorization.{field}[{index}]");
        match scope.as_str() {
            Some(scope)
                if !scope.contains('*')
                    && scope.ends_with(suffix)
                    && seen.insert(scope.to_owned()) => {}
            _ => issues.push(ValidationIssue::new(
                "AUTHORIZATION_SCOPE_NOT_EXPLICIT",
                path,
            )),
        }
    }
}

pub fn validate_manifest(value: &Value) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if !value.is_object() {
        return vec![ValidationIssue::new("ROOT_NOT_OBJECT", "$")];
    }
    reject_unknown_keys(
        value,
        &[
            "schema_version",
            "manifest_id",
            "composition",
            "participants",
            "relationships",
            "mappings",
            "authorization",
            "decision_scope",
            "qualification_profile_ref",
            "evaluation_mode",
            "claim_ceiling",
            "standard_conformity_claimed",
            "unknown_extension_policy",
        ],
        "$",
        &mut issues,
    );
    if string_at(value, &["schema_version"])
        != Some("luminous.industry-twin-interface-manifest.v0.1")
    {
        issues.push(ValidationIssue::new("SCHEMA_VERSION_UNSUPPORTED", "schema_version"));
    }
    if value["standard_conformity_claimed"].as_bool() != Some(false) {
        issues.push(ValidationIssue::new(
            "STANDARD_CONFORMITY_CLAIM_FORBIDDEN",
            "standard_conformity_claimed",
        ));
    }
    if string_at(value, &["evaluation_mode"]) != Some("synthetic_fixture")
        || string_at(value, &["claim_ceiling"]) != Some("synthetic_fixture")
    {
        issues.push(ValidationIssue::new(
            "CLAIM_CEILING_EXCEEDS_FIXTURE",
            "claim_ceiling",
        ));
    }
    if string_at(value, &["unknown_extension_policy"])
        != Some("preserve_opaque_or_report_loss")
    {
        issues.push(ValidationIssue::new(
            "UNKNOWN_EXTENSION_POLICY_UNSAFE",
            "unknown_extension_policy",
        ));
    }
    if string_at(value, &["qualification_profile_ref"]).is_none_or(str::is_empty) {
        issues.push(ValidationIssue::new(
            "QUALIFICATION_PROFILE_REFERENCE_MISSING",
            "qualification_profile_ref",
        ));
    }

    let participants = match array_at(value, "participants") {
        Some(items) if !items.is_empty() => items,
        _ => {
            issues.push(ValidationIssue::new("PARTICIPANTS_MISSING", "participants"));
            return issues;
        }
    };
    let mut participants_by_id: HashMap<String, &Value> = HashMap::new();
    let mut owner_ids = HashSet::new();
    for (index, participant) in participants.iter().enumerate() {
        let base = format!("participants[{index}]");
        reject_unknown_keys(
            participant,
            &[
                "participant_id",
                "system_id",
                "twin_id",
                "owner_id",
                "role",
                "native_model",
                "interface",
                "source_snapshot",
                "time_semantics",
            ],
            &base,
            &mut issues,
        );
        reject_unknown_keys(
            &participant["native_model"],
            &["model_id", "version", "artifact_sha256"],
            &format!("{base}.native_model"),
            &mut issues,
        );
        reject_unknown_keys(
            &participant["interface"],
            &[
                "standard_id",
                "edition",
                "document_kind",
                "protocol",
                "protocol_version",
                "adapter_id",
                "adapter_version",
                "native_schema",
            ],
            &format!("{base}.interface"),
            &mut issues,
        );
        reject_unknown_keys(
            &participant["interface"]["native_schema"],
            &["format", "schema_id", "schema_version", "schema_sha256"],
            &format!("{base}.interface.native_schema"),
            &mut issues,
        );
        reject_unknown_keys(
            &participant["source_snapshot"],
            &[
                "source_uri",
                "immutable_revision",
                "content_sha256",
                "retrieved_at",
                "rights_state",
                "license_or_basis",
            ],
            &format!("{base}.source_snapshot"),
            &mut issues,
        );
        reject_unknown_keys(
            &participant["time_semantics"],
            &[
                "clock_domain",
                "event_time_field",
                "ingestion_time_field",
                "sequence_field",
                "sequence_order",
                "timezone",
                "timestamp_precision",
            ],
            &format!("{base}.time_semantics"),
            &mut issues,
        );
        for (field, path) in [
            ("native_model", "native_model"),
            ("native_schema", "interface.native_schema"),
        ] {
            let digest_path = match field {
                "native_model" => &participant["native_model"]["artifact_sha256"],
                _ => &participant["interface"]["native_schema"]["schema_sha256"],
            };
            if digest_path
                .as_str()
                .is_none_or(|digest| !valid_sha256(digest))
            {
                issues.push(ValidationIssue::new(
                    "ARTIFACT_DIGEST_INVALID",
                    format!("{base}.{path}"),
                ));
            }
        }
        let Some(participant_id) = string_at(participant, &["participant_id"])
            .filter(|id| !id.is_empty())
        else {
            issues.push(ValidationIssue::new(
                "PARTICIPANT_ID_MISSING",
                format!("{base}.participant_id"),
            ));
            continue;
        };
        if participants_by_id
            .insert(participant_id.to_owned(), participant)
            .is_some()
        {
            issues.push(ValidationIssue::new(
                "PARTICIPANT_ID_DUPLICATE",
                format!("{base}.participant_id"),
            ));
        }
        if let Some(owner) = string_at(participant, &["owner_id"]).filter(|id| !id.is_empty()) {
            owner_ids.insert(owner.to_owned());
        } else {
            issues.push(ValidationIssue::new(
                "PARTICIPANT_OWNER_MISSING",
                format!("{base}.owner_id"),
            ));
        }

        let source = &participant["source_snapshot"];
        let revision = string_at(source, &["immutable_revision"]);
        if revision.is_none_or(str::is_empty) {
            issues.push(ValidationIssue::new(
                "SOURCE_REVISION_MISSING",
                format!("{base}.source_snapshot.immutable_revision"),
            ));
        }
        match string_at(source, &["content_sha256"]) {
            Some(digest) if valid_sha256(digest) => {}
            _ => issues.push(ValidationIssue::new(
                "SOURCE_DIGEST_INVALID",
                format!("{base}.source_snapshot.content_sha256"),
            )),
        }
        if string_at(source, &["source_uri"]).is_none_or(str::is_empty) {
            issues.push(ValidationIssue::new(
                "SOURCE_LOCATOR_MISSING",
                format!("{base}.source_snapshot.source_uri"),
            ));
        }
        match string_at(source, &["rights_state"]) {
            Some("verified" | "not_applicable") => {}
            Some("restricted") => issues.push(ValidationIssue::new(
                "SOURCE_RIGHTS_RESTRICTED",
                format!("{base}.source_snapshot.rights_state"),
            )),
            _ => issues.push(ValidationIssue::new(
                "SOURCE_RIGHTS_UNVERIFIED",
                format!("{base}.source_snapshot.rights_state"),
            )),
        }

        let time = &participant["time_semantics"];
        let event_field = string_at(time, &["event_time_field"]);
        let ingestion_field = string_at(time, &["ingestion_time_field"]);
        if event_field.is_none_or(str::is_empty)
            || ingestion_field.is_none_or(str::is_empty)
            || event_field == ingestion_field
        {
            issues.push(ValidationIssue::new(
                "SOURCE_AND_INGESTION_TIME_NOT_DISTINCT",
                format!("{base}.time_semantics"),
            ));
        }
        if string_at(time, &["clock_domain"]).is_none_or(str::is_empty) {
            issues.push(ValidationIssue::new(
                "SOURCE_CLOCK_DOMAIN_MISSING",
                format!("{base}.time_semantics.clock_domain"),
            ));
        }
        match string_at(time, &["sequence_order"]) {
            Some("strict_monotonic" | "monotonic") => {
                if string_at(time, &["sequence_field"]).is_none_or(str::is_empty) {
                    issues.push(ValidationIssue::new(
                        "SEQUENCE_FIELD_MISSING",
                        format!("{base}.time_semantics.sequence_field"),
                    ));
                }
            }
            Some("none") => {
                if string_at(time, &["sequence_field"]).is_some() {
                    issues.push(ValidationIssue::new(
                        "SEQUENCE_FIELD_WITH_NONE_ORDER",
                        format!("{base}.time_semantics.sequence_field"),
                    ));
                }
            }
            Some("unordered") => {}
            _ => issues.push(ValidationIssue::new(
                "SEQUENCE_ORDER_UNSPECIFIED",
                format!("{base}.time_semantics.sequence_order"),
            )),
        }
        if string_at(time, &["timestamp_precision"]) == Some("unknown") {
            issues.push(ValidationIssue::new(
                "SOURCE_TIMESTAMP_PRECISION_UNKNOWN",
                format!("{base}.time_semantics.timestamp_precision"),
            ));
        }
    }

    let composition = &value["composition"];
    reject_unknown_keys(
        composition,
        &[
            "mode",
            "composition_id",
            "federation_id",
            "member_participant_ids",
            "shared_schema_ref",
            "description",
        ],
        "composition",
        &mut issues,
    );
    let mode = string_at(composition, &["mode"]);
    let members = array_at(composition, "member_participant_ids");
    let mut member_ids = HashSet::new();
    if let Some(members) = members {
        for (index, member) in members.iter().enumerate() {
            if let Some(id) = member.as_str() {
                if !member_ids.insert(id.to_owned()) {
                    issues.push(ValidationIssue::new(
                        "COMPOSITION_MEMBER_DUPLICATE",
                        format!("composition.member_participant_ids[{index}]"),
                    ));
                }
                if !participants_by_id.contains_key(id) {
                    issues.push(ValidationIssue::new(
                        "COMPOSITION_MEMBER_UNKNOWN",
                        format!("composition.member_participant_ids[{index}]"),
                    ));
                }
            } else {
                issues.push(ValidationIssue::new(
                    "COMPOSITION_MEMBER_INVALID",
                    format!("composition.member_participant_ids[{index}]"),
                ));
            }
        }
    } else {
        issues.push(ValidationIssue::new(
            "COMPOSITION_MEMBERS_MISSING",
            "composition.member_participant_ids",
        ));
    }
    let participant_ids: HashSet<String> = participants_by_id.keys().cloned().collect();
    if member_ids != participant_ids {
        issues.push(ValidationIssue::new(
            "COMPOSITION_MEMBERSHIP_NOT_CLOSED",
            "composition.member_participant_ids",
        ));
    }
    match mode {
        Some("federated") if owner_ids.len() < 2 => issues.push(ValidationIssue::new(
            "FEDERATION_AUTHORITY_NOT_DISTRIBUTED",
            "participants.owner_id",
        )),
        Some("integrated")
            if string_at(composition, &["shared_schema_ref"]).is_none_or(str::is_empty) =>
        {
            issues.push(ValidationIssue::new(
                "INTEGRATED_SHARED_SCHEMA_MISSING",
                "composition.shared_schema_ref",
            ))
        }
        Some("integrated" | "unified" | "federated") => {}
        _ => issues.push(ValidationIssue::new(
            "COMPOSITION_MODE_UNSUPPORTED",
            "composition.mode",
        )),
    }

    let mut relationship_ids = HashSet::new();
    if let Some(relationships) = array_at(value, "relationships") {
        for (index, relationship) in relationships.iter().enumerate() {
            let base = format!("relationships[{index}]");
            reject_unknown_keys(
                relationship,
                &[
                    "relationship_id",
                    "from_participant_id",
                    "to_participant_id",
                    "relation",
                    "join_strategy",
                    "from_reference_field",
                    "to_reference_field",
                    "identity_namespace",
                ],
                &base,
                &mut issues,
            );
            if let Some(id) = string_at(relationship, &["relationship_id"]) {
                if !relationship_ids.insert(id.to_owned()) {
                    issues.push(ValidationIssue::new(
                        "RELATIONSHIP_ID_DUPLICATE",
                        format!("{base}.relationship_id"),
                    ));
                }
            }
            let from = string_at(relationship, &["from_participant_id"]);
            let to = string_at(relationship, &["to_participant_id"]);
            if from.is_none_or(|id| !participants_by_id.contains_key(id)) {
                issues.push(ValidationIssue::new(
                    "RELATIONSHIP_SOURCE_UNKNOWN",
                    format!("{base}.from_participant_id"),
                ));
            }
            if to.is_none_or(|id| !participants_by_id.contains_key(id)) {
                issues.push(ValidationIssue::new(
                    "RELATIONSHIP_TARGET_UNKNOWN",
                    format!("{base}.to_participant_id"),
                ));
            }
            if from.is_some() && from == to {
                issues.push(ValidationIssue::new(
                    "RELATIONSHIP_SELF_JOIN_FORBIDDEN",
                    base.clone(),
                ));
            }
            if !matches!(
                string_at(relationship, &["join_strategy"]),
                Some("exact_native_id" | "typed_reference")
            ) {
                issues.push(ValidationIssue::new(
                    "QUALIFIED_JOIN_STRATEGY_UNSUPPORTED",
                    format!("{base}.join_strategy"),
                ));
            }
            if string_at(relationship, &["identity_namespace"]).is_none_or(str::is_empty) {
                issues.push(ValidationIssue::new(
                    "RELATIONSHIP_IDENTITY_NAMESPACE_MISSING",
                    format!("{base}.identity_namespace"),
                ));
            }
        }
    }

    let mut mapping_ids = HashSet::new();
    if let Some(mappings) = array_at(value, "mappings") {
        for (index, mapping) in mappings.iter().enumerate() {
            let base = format!("mappings[{index}]");
            reject_unknown_keys(
                mapping,
                &[
                    "mapping_id",
                    "source_participant_id",
                    "source_field",
                    "target_concept",
                    "source_unit",
                    "target_unit",
                    "transform",
                    "loss_class",
                    "human_reviewed",
                    "loss_description",
                ],
                &base,
                &mut issues,
            );
            reject_unknown_keys(
                &mapping["transform"],
                &["transform_id", "version", "kind", "sha256"],
                &format!("{base}.transform"),
                &mut issues,
            );
            if let Some(id) = string_at(mapping, &["mapping_id"]) {
                if !mapping_ids.insert(id.to_owned()) {
                    issues.push(ValidationIssue::new(
                        "MAPPING_ID_DUPLICATE",
                        format!("{base}.mapping_id"),
                    ));
                }
            }
            let source_participant = string_at(mapping, &["source_participant_id"]);
            if source_participant.is_none_or(|id| !participants_by_id.contains_key(id)) {
                issues.push(ValidationIssue::new(
                    "MAPPING_SOURCE_UNKNOWN",
                    format!("{base}.source_participant_id"),
                ));
            }
            let source_unit = string_at(mapping, &["source_unit"]);
            let target_unit = string_at(mapping, &["target_unit"]);
            let transform_kind = string_at(mapping, &["transform", "kind"]);
            let transform_digest = string_at(mapping, &["transform", "sha256"]);
            if transform_digest.is_none_or(|digest| !valid_sha256(digest)) {
                issues.push(ValidationIssue::new(
                    "TRANSFORM_DIGEST_INVALID",
                    format!("{base}.transform.sha256"),
                ));
            }
            if source_unit != target_unit
                && !matches!(
                    transform_kind,
                    Some("unit_conversion" | "semantic_mapping" | "aggregation" | "time_mapping")
                )
            {
                issues.push(ValidationIssue::new(
                    "UNIT_CHANGE_WITHOUT_EXPLICIT_TRANSFORM",
                    base.clone(),
                ));
            }
            if transform_kind == Some("identity") && source_unit != target_unit {
                issues.push(ValidationIssue::new(
                    "IDENTITY_TRANSFORM_UNIT_MISMATCH",
                    base.clone(),
                ));
            }
            if string_at(mapping, &["loss_class"]) == Some("lossy")
                && mapping["human_reviewed"].as_bool() != Some(true)
            {
                issues.push(ValidationIssue::new(
                    "LOSSY_MAPPING_NOT_REVIEWED",
                    format!("{base}.human_reviewed"),
                ));
            }
            if string_at(mapping, &["loss_class"]) == Some("opaque")
                && string_at(mapping, &["loss_description"]).is_none_or(str::is_empty)
            {
                issues.push(ValidationIssue::new(
                    "OPAQUE_MAPPING_LOSS_UNDECLARED",
                    format!("{base}.loss_description"),
                ));
            }
        }
    }

    let authorization = &value["authorization"];
    reject_unknown_keys(
        authorization,
        &[
            "principal_id",
            "read_scopes",
            "write_scopes",
            "actuation_scopes",
            "authority_ref",
            "scope_separation_declared",
        ],
        "authorization",
        &mut issues,
    );
    if authorization["scope_separation_declared"].as_bool() != Some(true) {
        issues.push(ValidationIssue::new(
            "AUTHORIZATION_SCOPE_SEPARATION_UNDECLARED",
            "authorization.scope_separation_declared",
        ));
    }
    valid_scope_list(authorization, "read_scopes", ":read", &mut issues);
    valid_scope_list(authorization, "write_scopes", ":write", &mut issues);
    valid_scope_list(authorization, "actuation_scopes", ":actuate", &mut issues);

    let decision = &value["decision_scope"];
    reject_unknown_keys(
        decision,
        &[
            "decision_tier",
            "requested_action",
            "human_approval_required",
            "actuation_enabled",
        ],
        "decision_scope",
        &mut issues,
    );
    let tier = string_at(decision, &["decision_tier"]);
    let action = string_at(decision, &["requested_action"]);
    if tier == Some("D0_descriptive")
        && !matches!(action, Some("view" | "annotate"))
    {
        issues.push(ValidationIssue::new(
            "D0_ACTION_ESCALATION",
            "decision_scope.requested_action",
        ));
    }
    if !matches!(tier, Some("D0_descriptive" | "D1_advisory")) {
        issues.push(ValidationIssue::new(
            "DECISION_TIER_UNSUPPORTED",
            "decision_scope.decision_tier",
        ));
    }
    if decision["human_approval_required"].as_bool() != Some(true) {
        issues.push(ValidationIssue::new(
            "HUMAN_APPROVAL_REQUIRED",
            "decision_scope.human_approval_required",
        ));
    }
    if decision["actuation_enabled"].as_bool() != Some(false)
        || array_at(authorization, "actuation_scopes").is_some_and(|scopes| !scopes.is_empty())
    {
        issues.push(ValidationIssue::new(
            "ACTUATION_NOT_ADMITTED_BY_ITIP_V0_1",
            "decision_scope.actuation_enabled",
        ));
    }

    issues.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.path.cmp(&right.path))
    });
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(source: &str) -> Value {
        serde_json::from_str(source).expect("synthetic fixture must parse")
    }

    fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
        issues.iter().any(|issue| issue.code == code)
    }

    #[test]
    fn unified_source_preserving_fixture_is_valid() {
        let value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        assert!(validate_manifest(&value).is_empty());
    }

    #[test]
    fn federated_fixture_requires_and_preserves_distinct_authorities() {
        let value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-federated-synthetic-v0.1.json"
        ));
        assert!(validate_manifest(&value).is_empty());
    }

    #[test]
    fn duplicate_participant_id_is_rejected() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["participants"][1]["participant_id"] = json!("pump-physical");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "PARTICIPANT_ID_DUPLICATE"));
        assert!(has_code(&issues, "COMPOSITION_MEMBERSHIP_NOT_CLOSED"));
    }

    #[test]
    fn dangling_relationship_is_rejected() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["relationships"][0]["to_participant_id"] = json!("missing-participant");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "RELATIONSHIP_TARGET_UNKNOWN"));
    }

    #[test]
    fn source_digest_and_revision_are_mandatory() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["participants"][0]["source_snapshot"]["content_sha256"] = json!("bad");
        value["participants"][0]["source_snapshot"]["immutable_revision"] = json!("");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "SOURCE_DIGEST_INVALID"));
        assert!(has_code(&issues, "SOURCE_REVISION_MISSING"));
    }

    #[test]
    fn event_time_cannot_be_overwritten_by_ingestion_time() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        let event_time = value["participants"][0]["time_semantics"]["event_time_field"].clone();
        value["participants"][0]["time_semantics"]["ingestion_time_field"] = event_time;
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "SOURCE_AND_INGESTION_TIME_NOT_DISTINCT"));
    }

    #[test]
    fn sequence_order_requires_a_sequence_field() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["participants"][0]["time_semantics"]
            .as_object_mut()
            .unwrap()
            .remove("sequence_field");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "SEQUENCE_FIELD_MISSING"));
    }

    #[test]
    fn qualified_join_cannot_use_fuzzy_matching() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["relationships"][0]["join_strategy"] = json!("fuzzy_match");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "QUALIFIED_JOIN_STRATEGY_UNSUPPORTED"));
    }

    #[test]
    fn unit_change_requires_explicit_transform() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["mappings"][0]["target_unit"] = json!("kPa");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "UNIT_CHANGE_WITHOUT_EXPLICIT_TRANSFORM"));
        assert!(has_code(&issues, "IDENTITY_TRANSFORM_UNIT_MISMATCH"));
    }

    #[test]
    fn lossy_mapping_requires_human_review() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["mappings"][0]["loss_class"] = json!("lossy");
        value["mappings"][0]["human_reviewed"] = json!(false);
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "LOSSY_MAPPING_NOT_REVIEWED"));
    }

    #[test]
    fn federated_mode_requires_distinct_authority_owners() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-federated-synthetic-v0.1.json"
        ));
        let owner = value["participants"][0]["owner_id"].clone();
        value["participants"][1]["owner_id"] = owner;
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "FEDERATION_AUTHORITY_NOT_DISTRIBUTED"));
    }

    #[test]
    fn unknown_rights_are_not_promoted_into_admitted_evidence() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["participants"][0]["source_snapshot"]["rights_state"] = json!("unknown");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "SOURCE_RIGHTS_UNVERIFIED"));
    }

    #[test]
    fn read_scope_cannot_smuggle_actuation_permission() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["authorization"]["read_scopes"][0] = json!("pump-001:actuate");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "AUTHORIZATION_SCOPE_NOT_EXPLICIT"));
    }

    #[test]
    fn d0_cannot_request_advisory_action() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["decision_scope"]["requested_action"] = json!("recommend");
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "D0_ACTION_ESCALATION"));
    }


    #[test]
    fn frozen_negative_fixture_files_are_rejected_by_semantic_validator() {
        let negative_fixtures = [
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-unknown-property-v0.1.json"
            ),
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-missing-digest-v0.1.json"
            ),
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-dangling-reference-v0.1.json"
            ),
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-clock-alias-v0.1.json"
            ),
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-fuzzy-join-v0.1.json"
            ),
            include_str!(
                "../../../schemas/examples/industry-twin-interface-negative-unit-transform-v0.1.json"
            )
        ];
        for fixture_text in negative_fixtures {
            let value: Value = serde_json::from_str(fixture_text).unwrap();
            assert!(!validate_manifest(&value).is_empty());
        }
    }

    #[test]
    fn unknown_nested_properties_are_rejected() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["participants"][0]["time_semantics"]["guessed_clock_offset_ms"] = json!(0);
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "UNKNOWN_PROPERTY_NOT_ALLOWED"));
    }

    #[test]
    fn standards_conformity_claim_is_forbidden() {
        let mut value = fixture(include_str!(
            "../../../schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json"
        ));
        value["standard_conformity_claimed"] = json!(true);
        let issues = validate_manifest(&value);
        assert!(has_code(&issues, "STANDARD_CONFORMITY_CLAIM_FORBIDDEN"));
    }
}
