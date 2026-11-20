use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs, process};

const FIXTURE_PATH: &str =
    "tools/shared-amenity-spatial-eval/fixtures/SERVICE_DEBT_METROLOGY_FIXTURES_V0.1.json";
const EXPECTED_SCHEMA: &str = "luminous.service-debt-metrology-fixtures";
const EXPECTED_VERSION: &str = "0.1.0";
const EXPECTED_MATURITY: &str = "SYNTHETIC_METROLOGY_ORACLE_ONLY";
const EXPECTED_CLAIM_CEILING: &str =
    "NO_LIVE_RESIDENT_DEMAND_DELIVERY_CAPACITY_UNEXPRESSED_NEED_OR_WELFARE_CLAIM";
const EXPECTED_CASE_COUNT: usize = 19;

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number"))?;
        Ok(UniqueValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(UniqueValue(value)) = sequence.next_element::<UniqueValue>()? {
            values.push(value);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate JSON object key: {key}")));
            }
            let UniqueValue(value) = map.next_value::<UniqueValue>()?;
            values.insert(key, value);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("missing or empty string field {key}"))
}

fn number(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or invalid non-negative integer field {key}"))
}

fn optional_number(value: &Value, key: &str) -> Result<Option<u64>, String> {
    match value.get(key) {
        Some(Value::Null) => Ok(None),
        Some(raw) => raw
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("invalid non-negative integer field {key}")),
        None => Err(format!("missing field {key}")),
    }
}

fn evidence_is_current(value: &Value) -> bool {
    value
        .get("evidence")
        .and_then(|evidence| evidence.get("freshness"))
        .and_then(Value::as_str)
        == Some("CURRENT")
        && value
            .get("evidence")
            .and_then(|evidence| evidence.get("revision"))
            .and_then(Value::as_str)
            .is_some_and(|revision| !revision.trim().is_empty())
}

fn validate_evidence(
    value: &Value,
    label: &str,
    expected_owner: &str,
) -> Result<(), String> {
    let evidence = value
        .get("evidence")
        .ok_or_else(|| format!("{label} is missing evidence"))?;
    if string(evidence, "owner")? != expected_owner {
        return Err(format!("{label} has an unexpected evidence owner"));
    }
    string(evidence, "record_id")?;
    string(evidence, "revision")?;
    match string(evidence, "freshness")? {
        "CURRENT" | "STALE" => Ok(()),
        _ => Err(format!("{label} has an invalid freshness state")),
    }
}

fn qualify_demand_or_capacity(
    value: &Value,
    common_scope: &Value,
    label: &str,
) -> Result<(String, Option<u64>), String> {
    if value.get("scope") != Some(common_scope) {
        return Ok(("NOT_COMPARABLE".to_owned(), None));
    }

    let (expected_owner, expected_basis) = match label {
        "demand" => ("MYCELIX_DEMAND", "OWNER_QUALIFIED_SERVICE_DEMAND"),
        "capacity" => (
            "SYMTROPY_CAPACITY",
            "OWNER_QUALIFIED_DELIVERABLE_CAPACITY",
        ),
        _ => return Err(format!("unsupported measurement owner for {label}")),
    };
    validate_evidence(value, label, expected_owner)?;
    if string(value, "basis")? != expected_basis {
        return Err(format!("{label} has an unexpected quantity basis"));
    }
    let state = string(value, "state")?;
    let coverage = string(value, "coverage")?;
    let quantity = optional_number(value, "quantity")?;

    if state != "KNOWN" && state != "UNKNOWN" {
        return Err(format!("{label} has an invalid state"));
    }
    if !["COMPLETE", "PARTIAL", "UNKNOWN"].contains(&coverage) {
        return Err(format!("{label} has invalid coverage"));
    }
    if state == "UNKNOWN" && quantity.is_some() {
        return Err(format!("{label} is UNKNOWN but contains a numeric quantity"));
    }
    if state == "KNOWN" && coverage == "COMPLETE" && evidence_is_current(value) {
        let quantity = quantity.ok_or_else(|| format!("{label} is KNOWN but has no quantity"))?;
        Ok(("KNOWN".to_owned(), Some(quantity)))
    } else {
        Ok(("UNKNOWN".to_owned(), None))
    }
}

fn capacity_gap(
    demand: &Value,
    capacity: &Value,
    common_scope: &Value,
) -> Result<(String, Option<u64>, Option<u64>), String> {
    let (demand_state, demand_units) =
        qualify_demand_or_capacity(demand, common_scope, "demand")?;
    let (capacity_state, capacity_units) =
        qualify_demand_or_capacity(capacity, common_scope, "capacity")?;

    if demand_state == "NOT_COMPARABLE" || capacity_state == "NOT_COMPARABLE" {
        return Ok(("NOT_COMPARABLE".to_owned(), None, None));
    }
    if demand_state != "KNOWN" || capacity_state != "KNOWN" {
        return Ok(("UNKNOWN".to_owned(), None, None));
    }

    let demand_units = demand_units.expect("known demand has units");
    let capacity_units = capacity_units.expect("known capacity has units");
    if capacity_units < demand_units {
        Ok((
            "KNOWN".to_owned(),
            Some(demand_units - capacity_units),
            Some(0),
        ))
    } else {
        Ok((
            "KNOWN".to_owned(),
            Some(0),
            Some(capacity_units - demand_units),
        ))
    }
}

fn accepted_undelivered(
    register: &Value,
    common_scope: &Value,
) -> Result<(String, Option<u64>), String> {
    if register.get("scope") != Some(common_scope) {
        return Ok(("NOT_COMPARABLE".to_owned(), None));
    }

    validate_evidence(
        register,
        "request register",
        "MYCELIX_REQUEST_REGISTER",
    )?;
    let coverage = string(register, "coverage")?;
    if coverage != "COMPLETE" || !evidence_is_current(register) {
        if !["PARTIAL", "UNKNOWN", "COMPLETE"].contains(&coverage) {
            return Err("request register has invalid coverage".to_owned());
        }
        return Ok(("UNKNOWN".to_owned(), None));
    }

    let records = register
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| "request register records must be an array".to_owned())?;

    let mut seen = BTreeSet::new();
    let mut total = 0_u64;
    let mut unresolved = false;

    for record in records {
        let request_id = string(record, "request_id")?;
        if !seen.insert(request_id.to_owned()) {
            return Err(format!("duplicate request identifier {request_id}"));
        }
        if record.get("scope") != Some(common_scope) {
            return Ok(("NOT_COMPARABLE".to_owned(), None));
        }
        validate_evidence(
            record,
            "request record",
            "MYCELIX_REQUEST_DELIVERY",
        )?;

        let request_state = string(record, "state")?;
        let delivery_state = string(record, "delivery_state")?;
        if !["ACCEPTED", "CANCELLED", "DECLINED"].contains(&request_state) {
            return Err(format!("request {request_id} has an invalid state"));
        }
        if !["KNOWN", "UNKNOWN"].contains(&delivery_state) {
            return Err(format!("request {request_id} has an invalid delivery state"));
        }

        let accepted = number(record, "accepted_units")?;
        let delivered = optional_number(record, "delivered_units")?;
        if request_state == "CANCELLED" || request_state == "DECLINED" {
            if accepted != 0 || delivered != Some(0) || delivery_state != "KNOWN" {
                return Err(format!(
                    "non-accepted request {request_id} must have zero accepted/delivered units"
                ));
            }
            if !evidence_is_current(record) {
                unresolved = true;
            }
            continue;
        }

        if accepted == 0 {
            return Err(format!("accepted request {request_id} has zero accepted units"));
        }
        match (delivery_state, delivered) {
            ("UNKNOWN", None) => {
                unresolved = true;
                continue;
            }
            ("KNOWN", Some(delivered)) => {
                if delivered > accepted {
                    return Err(format!(
                        "request {request_id} delivery exceeds accepted units"
                    ));
                }
                if !evidence_is_current(record) {
                    unresolved = true;
                    continue;
                }
                total = total
                    .checked_add(accepted - delivered)
                    .ok_or_else(|| "accepted-undelivered quantity overflow".to_owned())?;
            }
            _ => {
                return Err(format!(
                    "request {request_id} has inconsistent delivery state and quantity"
                ));
            }
        }
    }

    if unresolved {
        Ok(("UNKNOWN".to_owned(), None))
    } else {
        Ok(("KNOWN".to_owned(), Some(total)))
    }
}

fn unexpressed_need(
    assessment: &Value,
    common_scope: &Value,
) -> Result<(String, Option<u64>, Option<u64>, Option<u64>), String> {
    if assessment.get("scope") != Some(common_scope) {
        return Ok(("NOT_COMPARABLE".to_owned(), None, None, None));
    }

    match string(assessment, "state")? {
        "NOT_COLLECTED" => Ok(("NOT_COLLECTED".to_owned(), None, None, None)),
        "UNKNOWN" => Ok(("UNKNOWN".to_owned(), None, None, None)),
        "MEASURED" => {
            validate_evidence(
                assessment,
                "resident need assessment",
                "RESIDENT_NEED_ASSESSMENT",
            )?;
            if !evidence_is_current(assessment) {
                return Ok(("UNKNOWN".to_owned(), None, None, None));
            }
            let method = string(assessment, "method")?;
            if method != "WEIGHTED_RESIDENT_SURVEY" {
                return Err("measured unexpressed need requires an approved resident-survey method".to_owned());
            }
            let estimate = optional_number(assessment, "estimate_units")?
                .ok_or_else(|| "measured need has no estimate".to_owned())?;
            let lower = optional_number(assessment, "lower_bound_units")?
                .ok_or_else(|| "measured need has no lower bound".to_owned())?;
            let upper = optional_number(assessment, "upper_bound_units")?
                .ok_or_else(|| "measured need has no upper bound".to_owned())?;
            let sample_size = number(assessment, "sample_size")?;
            let confidence = number(assessment, "confidence_level_basis_points")?;
            if sample_size == 0 || confidence == 0 || confidence >= 10_000 {
                return Err("resident need sample size or confidence level is invalid".to_owned());
            }
            if lower > estimate || estimate > upper {
                return Err("resident need estimate lies outside its uncertainty interval".to_owned());
            }
            Ok(("MEASURED".to_owned(), Some(estimate), Some(lower), Some(upper)))
        }
        _ => Err("resident need assessment has an invalid state".to_owned()),
    }
}

fn evaluate_case(case: &Value) -> Result<Value, String> {
    let case_id = string(case, "case_id")?;
    let common_scope = case
        .get("scope")
        .ok_or_else(|| format!("case {case_id} has no common scope"))?;
    let start = number(common_scope, "start_seconds")?;
    let end = number(common_scope, "end_seconds")?;
    if end <= start {
        return Err(format!("case {case_id} has a non-positive interval"));
    }
    string(common_scope, "scope_digest")?;
    string(common_scope, "population_digest")?;
    string(common_scope, "cohort_id")?;
    string(common_scope, "service_id")?;
    string(common_scope, "functional_unit_id")?;
    string(common_scope, "unit_id")?;
    string(common_scope, "period_id")?;
    string(common_scope, "time_basis_id")?;

    let (gap_state, shortfall, surplus) = capacity_gap(
        case.get("demand").ok_or("case missing demand")?,
        case.get("capacity").ok_or("case missing capacity")?,
        common_scope,
    )?;
    let (request_state, undelivered) = accepted_undelivered(
        case.get("request_register")
            .ok_or("case missing request_register")?,
        common_scope,
    )?;
    let (need_state, need_estimate, need_lower, need_upper) = unexpressed_need(
        case.get("need_assessment")
            .ok_or("case missing need_assessment")?,
        common_scope,
    )?;

    let mut reasons = Vec::new();
    match gap_state.as_str() {
        "UNKNOWN" => reasons.push("CAPACITY_GAP_EVIDENCE_UNRESOLVED"),
        "NOT_COMPARABLE" => reasons.push("CAPACITY_GAP_SCOPE_MISMATCH"),
        _ => {}
    }
    match request_state.as_str() {
        "UNKNOWN" => reasons.push("REQUEST_DELIVERY_EVIDENCE_UNRESOLVED"),
        "NOT_COMPARABLE" => reasons.push("REQUEST_DELIVERY_SCOPE_MISMATCH"),
        _ => {}
    }
    match need_state.as_str() {
        "NOT_COLLECTED" => reasons.push("RESIDENT_NEED_NOT_ASSESSED"),
        "UNKNOWN" => reasons.push("RESIDENT_NEED_EVIDENCE_UNRESOLVED"),
        "NOT_COMPARABLE" => reasons.push("RESIDENT_NEED_SCOPE_MISMATCH"),
        _ => {}
    }

    let metrics = json!({
        "capacity_gap_state": gap_state,
        "capacity_shortfall_units": shortfall,
        "capacity_surplus_units": surplus,
        "accepted_undelivered_state": request_state,
        "accepted_undelivered_units": undelivered,
        "unexpressed_need_state": need_state,
        "unexpressed_need_estimate_units": need_estimate,
        "unexpressed_need_lower_bound_units": need_lower,
        "unexpressed_need_upper_bound_units": need_upper,
        "unresolved_exposure_state": if reasons.is_empty() { "NONE" } else { "PRESENT" },
        "unresolved_reason_codes": reasons,
        "unresolved_duration_seconds": if reasons.is_empty() { 0 } else { end - start }
    });

    let expected = case
        .get("expected")
        .ok_or_else(|| format!("case {case_id} has no expected result"))?;
    if &metrics != expected {
        return Err(format!(
            "frozen oracle mismatch for {case_id}: expected {expected}, computed {metrics}"
        ));
    }

    Ok(json!({
        "case_id": case_id,
        "scope": common_scope,
        "metrics": metrics
    }))
}

fn evaluate(bytes: &[u8]) -> Result<Value, String> {
    let UniqueValue(fixture) = serde_json::from_slice(bytes)
        .map_err(|error| format!("invalid or ambiguous JSON: {error}"))?;

    if string(&fixture, "schema")? != EXPECTED_SCHEMA
        || string(&fixture, "schema_version")? != EXPECTED_VERSION
        || string(&fixture, "maturity")? != EXPECTED_MATURITY
        || string(&fixture, "claim_ceiling")? != EXPECTED_CLAIM_CEILING
    {
        return Err("fixture identity, version, maturity or claim ceiling changed".to_owned());
    }

    let policy = fixture
        .get("policy")
        .ok_or_else(|| "fixture has no policy".to_owned())?;
    for key in [
        "no_cross_metric_scalar",
        "unknown_is_not_zero",
        "booking_is_not_delivery",
        "no_inference_from_absent_requests",
    ] {
        if policy.get(key) != Some(&Value::Bool(true)) {
            return Err(format!("fixture policy {key} must remain true"));
        }
    }
    if string(policy, "capacity_gap_basis")?
        != "SAME_SCOPE_COMPLETE_DEMAND_AND_DELIVERABLE_CAPACITY_FOR_ONE_INTERVAL"
        || string(policy, "request_debt_basis")?
            != "IDENTITY_LINKED_ACCEPTED_REQUEST_MINUS_COMPLETED_DELIVERY"
        || string(policy, "unexpressed_need_basis")?
            != "INDEPENDENT_RESIDENT_NEED_ASSESSMENT_ONLY"
    {
        return Err("fixture policy semantics changed".to_owned());
    }

    let cases = fixture
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| "fixture cases must be an array".to_owned())?;
    if cases.len() != EXPECTED_CASE_COUNT {
        return Err(format!(
            "expected {EXPECTED_CASE_COUNT} cases, found {}",
            cases.len()
        ));
    }

    let mut seen = BTreeSet::new();
    let mut results = Vec::with_capacity(cases.len());
    for case in cases {
        let case_id = string(case, "case_id")?;
        if !seen.insert(case_id.to_owned()) {
            return Err(format!("duplicate case identifier {case_id}"));
        }
        results.push(evaluate_case(case)?);
    }

    let digest = Sha256::digest(bytes);
    let fixture_sha256 = format!("{digest:x}");
    Ok(json!({
        "schema": "luminous.service-debt-metrology-evaluation",
        "schema_version": "0.1.0",
        "maturity": EXPECTED_MATURITY,
        "claim_ceiling": EXPECTED_CLAIM_CEILING,
        "fixture_sha256": fixture_sha256,
        "case_count": results.len(),
        "oracle_match_count": results.len(),
        "results": results
    }))
}

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| FIXTURE_PATH.to_owned());
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("cannot read fixture {path}: {error}");
        process::exit(2);
    });
    match evaluate(&bytes) {
        Ok(output) => println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("evaluation output must serialize")
        ),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../fixtures/SERVICE_DEBT_METROLOGY_FIXTURES_V0.1.json");

    fn fixture_value() -> Value {
        serde_json::from_str(FIXTURE).expect("fixture JSON should parse")
    }

    #[test]
    fn all_nineteen_hand_authored_golden_cases_match() {
        let output = evaluate(FIXTURE.as_bytes()).expect("frozen oracle should match");
        assert_eq!(output["case_count"], 19);
        assert_eq!(output["oracle_match_count"], 19);
    }

    #[test]
    fn absent_requests_do_not_become_unexpressed_need_zero() {
        let output = evaluate(FIXTURE.as_bytes()).expect("frozen oracle should match");
        let row = output["results"]
            .as_array()
            .expect("results array")
            .iter()
            .find(|row| {
                row["case_id"] == "NO_REQUESTS_WITHOUT_NEED_ASSESSMENT_DO_NOT_MEAN_ZERO_NEED"
            })
            .expect("no-request case");
        assert_eq!(row["metrics"]["unexpressed_need_state"], "NOT_COLLECTED");
        assert_eq!(row["metrics"]["unexpressed_need_estimate_units"], Value::Null);
    }

    #[test]
    fn capacity_shortfall_is_not_substituted_for_request_delivery_debt() {
        let output = evaluate(FIXTURE.as_bytes()).expect("frozen oracle should match");
        let row = output["results"]
            .as_array()
            .expect("results array")
            .iter()
            .find(|row| row["case_id"] == "CAPACITY_SHORTFALL_IS_NOT_REQUEST_DEBT")
            .expect("capacity shortfall case");
        assert_eq!(row["metrics"]["capacity_shortfall_units"], 3);
        assert_eq!(row["metrics"]["accepted_undelivered_units"], 0);
    }

    #[test]
    fn accepted_delivery_cannot_exceed_the_accepted_quantity() {
        let mut value = fixture_value();
        let case = value["cases"]
            .as_array_mut()
            .expect("case array")
            .iter_mut()
            .find(|case| case["case_id"] == "PARTIALLY_DELIVERED_ACCEPTED_REQUEST")
            .expect("request case");
        case["request_register"]["records"][0]["delivered_units"] = Value::from(7_u64);
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "over-delivery must fail closed rather than create negative debt"
        );
    }

    #[test]
    fn duplicate_request_identifiers_are_rejected() {
        let mut value = fixture_value();
        let case = value["cases"]
            .as_array_mut()
            .expect("case array")
            .iter_mut()
            .find(|case| case["case_id"] == "MULTIPLE_PARTIAL_DELIVERIES_SUM_ONCE_PER_REQUEST")
            .expect("multi-request case");
        let duplicate = case["request_register"]["records"][0].clone();
        case["request_register"]["records"]
            .as_array_mut()
            .expect("request records")
            .push(duplicate);
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "duplicate request IDs must not double count service"
        );
    }

    #[test]
    fn unexpected_capacity_quantity_basis_is_rejected() {
        let mut value = fixture_value();
        value["cases"][0]["capacity"]["basis"] =
            Value::String("NAMEPLATE_CAPACITY".to_owned());
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "nominal capacity must not be treated as deliverable capacity"
        );
    }

    #[test]
    fn unexpected_demand_owner_is_rejected() {
        let mut value = fixture_value();
        value["cases"][0]["demand"]["evidence"]["owner"] =
            Value::String("UNVERIFIED_FEED".to_owned());
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "unknown evidence owners must not be promoted to KNOWN"
        );
    }

    #[test]
    fn unknown_capacity_cannot_carry_a_numeric_quantity() {
        let mut value = fixture_value();
        value["cases"][0]["capacity"]["state"] = Value::String("UNKNOWN".to_owned());
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "unknown capacity cannot silently retain a numeric value"
        );
    }

    #[test]
    fn unknown_delivery_cannot_carry_a_delivered_quantity() {
        let mut value = fixture_value();
        let case = value["cases"]
            .as_array_mut()
            .expect("case array")
            .iter_mut()
            .find(|case| case["case_id"] == "PARTIALLY_DELIVERED_ACCEPTED_REQUEST")
            .expect("partial delivery case");
        case["request_register"]["records"][0]["delivery_state"] =
            Value::String("UNKNOWN".to_owned());
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "unknown delivery cannot also assert a completed quantity"
        );
    }

    #[test]
    fn non_accepted_request_cannot_carry_accepted_or_delivered_units() {
        let mut value = fixture_value();
        let case = value["cases"]
            .as_array_mut()
            .expect("case array")
            .iter_mut()
            .find(|case| {
                case["case_id"] == "CANCELLED_AND_DECLINED_REQUESTS_ARE_NOT_UNDELIVERED_SERVICE"
            })
            .expect("cancelled request case");
        case["request_register"]["records"][0]["accepted_units"] = Value::from(1_u64);
        assert!(
            evaluate(value.to_string().as_bytes()).is_err(),
            "cancelled/declined requests cannot create accepted service quantities"
        );
    }

    #[test]
    fn duplicate_json_object_keys_are_rejected() {
        let ambiguous = br#"{"schema":"a","schema":"b"}"#;
        assert!(evaluate(ambiguous).is_err());
    }
}
