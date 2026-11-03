//! Executable conformance checks for the synthetic Sol-Atlas source-record contract.
//!
//! These checks validate the declared JSON Schema dialect and the contract's key
//! cross-field invariants. They do not authenticate publishers, grant source rights,
//! or establish operational/legal applicability.

#[cfg(test)]
mod tests {
    use jsonschema::draft202012;
    use serde_json::Value;
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    const SCHEMA_JSON: &str =
        include_str!("../../../schemas/aviation/airspace-source-record.schema.json");
    const FIXTURES_JSON: &str =
        include_str!("../../../tests/fixtures/aviation/airspace-source-record-fixtures.json");

    fn schema() -> Value {
        serde_json::from_str(SCHEMA_JSON).expect("schema must be valid JSON")
    }

    fn fixture_cases() -> Vec<(String, Value)> {
        let suite: Value =
            serde_json::from_str(FIXTURES_JSON).expect("fixture suite must be valid JSON");
        suite["cases"]
            .as_array()
            .expect("fixture suite must contain cases")
            .iter()
            .map(|case| {
                (
                    case["case_id"]
                        .as_str()
                        .expect("fixture case must have an id")
                        .to_owned(),
                    case["record"].clone(),
                )
            })
            .collect()
    }

    fn fixture(case_id: &str) -> Value {
        fixture_cases()
            .into_iter()
            .find(|(id, _)| id == case_id)
            .unwrap_or_else(|| panic!("missing fixture case {case_id}"))
            .1
    }

    fn text<'a>(record: &'a Value, pointer: &str) -> Option<&'a str> {
        record.pointer(pointer).and_then(Value::as_str)
    }

    fn parse_instant(
        record: &Value,
        pointer: &str,
        issues: &mut Vec<String>,
    ) -> Option<OffsetDateTime> {
        let raw = record.pointer(pointer)?.as_str()?;
        match OffsetDateTime::parse(raw, &Rfc3339) {
            Ok(value) => Some(value),
            Err(error) => {
                issues.push(format!("{pointer} is not parseable RFC 3339: {error}"));
                None
            }
        }
    }

    fn semantic_issues(record: &Value) -> Vec<String> {
        let mut issues = Vec::new();

        let input_freshness = text(record, "/freshness/status");
        let assessed_freshness = text(record, "/assessment/freshness_for_use");
        if input_freshness != assessed_freshness {
            issues.push(
                "assessment freshness must match the result of the declared freshness policy"
                    .to_owned(),
            );
        }
        if input_freshness != Some("current_for_declared_use")
            && assessed_freshness == Some("current_for_declared_use")
        {
            issues.push(
                "unknown, stale, or unevaluated source freshness cannot satisfy a current-use gate"
                    .to_owned(),
            );
        }

        let coverage_states = [
            text(record, "/coverage/geographic"),
            text(record, "/coverage/temporal"),
            text(record, "/coverage/attributes"),
        ];
        if text(record, "/assessment/coverage") == Some("complete_for_declared_scope")
            && coverage_states.iter().any(|state| *state != Some("known"))
        {
            issues.push(
                "complete coverage cannot be claimed while geographic, temporal, or attribute coverage is not known"
                    .to_owned(),
            );
        }

        let conflict = text(record, "/assessment/conflict");
        let applicability = text(record, "/assessment/applicability");
        if (conflict == Some("conflicted")) != (applicability == Some("conflicted")) {
            issues.push(
                "an unresolved applicable-source conflict must be reflected in applicability"
                    .to_owned(),
            );
        }

        if text(record, "/source/authentication/state") == Some("failed")
            && text(record, "/assessment/source_authenticity") == Some("authenticated")
        {
            issues.push(
                "failed source authentication cannot be assessed as authenticated".to_owned(),
            );
        }
        if text(record, "/assessment/source_authenticity") == Some("authenticated")
            && !matches!(
                text(record, "/source/authentication/state"),
                Some("publisher_authenticated" | "signature_verified")
            )
        {
            issues.push(
                "authenticated assessment requires declared publisher or signature evidence"
                    .to_owned(),
            );
        }

        if record
            .pointer("/validity/withdrawn")
            .and_then(Value::as_bool)
            == Some(true)
            && applicability == Some("applicable")
        {
            issues.push("withdrawn source claims cannot remain applicable".to_owned());
        }

        for (from_pointer, until_pointer, label) in [
            (
                "/validity/effective_from",
                "/validity/effective_until",
                "source validity interval",
            ),
            (
                "/scope/valid_time_range/from",
                "/scope/valid_time_range/until",
                "declared scope interval",
            ),
        ] {
            let from = parse_instant(record, from_pointer, &mut issues);
            let until = parse_instant(record, until_pointer, &mut issues);
            match (from, until) {
                (Some(from), Some(until)) if from >= until => {
                    issues.push(format!("{label} must be strictly increasing"));
                }
                _ => {}
            }
        }

        let lower = record
            .pointer("/scope/altitude_range/lower")
            .and_then(Value::as_f64);
        let upper = record
            .pointer("/scope/altitude_range/upper")
            .and_then(Value::as_f64);
        if matches!((lower, upper), (Some(low), Some(high)) if low > high) {
            issues.push("altitude range lower bound cannot exceed its upper bound".to_owned());
        }

        issues
    }

    #[test]
    fn schema_is_valid_against_the_draft_2020_12_meta_schema() {
        let schema = schema();
        draft202012::meta::validate(&schema)
            .expect("source-record schema must satisfy the Draft 2020-12 meta-schema");
    }

    #[test]
    fn all_synthetic_fixture_records_pass_schema_and_semantic_checks() {
        let schema = schema();
        let validator = draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("schema must compile as Draft 2020-12");

        let cases = fixture_cases();
        assert_eq!(cases.len(), 4, "fixture suite must cover four named cases");
        for (case_id, record) in cases {
            validator
                .validate(&record)
                .unwrap_or_else(|error| panic!("{case_id} failed schema validation: {error}"));
            let issues = semantic_issues(&record);
            assert!(
                issues.is_empty(),
                "{case_id} failed semantic validation: {issues:?}"
            );
        }
    }

    #[test]
    fn declared_crs_requires_an_identifier_and_digest_must_be_sha256_syntax() {
        let schema = schema();
        let validator = draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("schema must compile");

        let mut no_crs_id = fixture("fresh-synthetic-record");
        no_crs_id["spatial_reference"]["horizontal_crs"]["identifier"] = Value::Null;
        assert!(validator.validate(&no_crs_id).is_err());

        let mut bad_digest = fixture("fresh-synthetic-record");
        bad_digest["source"]["artifact"]["sha256"] = Value::String("xyz".to_owned());
        assert!(validator.validate(&bad_digest).is_err());
    }

    #[test]
    fn stale_data_cannot_be_promoted_to_current_without_changing_its_policy_result() {
        let mut stale = fixture("stale-record-does-not-satisfy-current-use");
        stale["assessment"]["freshness_for_use"] =
            Value::String("current_for_declared_use".to_owned());
        let issues = semantic_issues(&stale);
        assert!(!issues.is_empty(), "mismatched freshness must be rejected");
    }

    #[test]
    fn incomplete_coverage_cannot_be_labelled_complete() {
        let mut unknown = fixture("unknown-coverage-is-not-unrestricted");
        unknown["assessment"]["coverage"] = Value::String("complete_for_declared_scope".to_owned());
        assert!(!semantic_issues(&unknown).is_empty());
    }

    #[test]
    fn conflicted_sources_cannot_be_labelled_applicable() {
        let mut conflicted = fixture("conflicted-record-remains-unresolved");
        conflicted["assessment"]["applicability"] = Value::String("applicable".to_owned());
        assert!(!semantic_issues(&conflicted).is_empty());
    }

    #[test]
    fn reversed_validity_interval_is_rejected() {
        let mut record = fixture("fresh-synthetic-record");
        record["validity"]["effective_from"] = Value::String("2026-10-22T00:00:00Z".to_owned());
        record["validity"]["effective_until"] = Value::String("2026-10-21T00:00:00Z".to_owned());
        assert!(!semantic_issues(&record).is_empty());
    }

    #[test]
    fn failed_source_authentication_cannot_be_promoted_to_authenticated() {
        let mut record = fixture("fresh-synthetic-record");
        record["source"]["authentication"]["state"] = Value::String("failed".to_owned());
        record["assessment"]["source_authenticity"] = Value::String("authenticated".to_owned());
        assert!(!semantic_issues(&record).is_empty());
    }
}
