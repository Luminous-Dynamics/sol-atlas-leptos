//! Evidence-admission reference oracle for the City heat/retrofit atlas.
//! Does not compute exposure, simulate buildings, estimate population or infer indoor temperature.

use serde_json::{Value, json};
use std::collections::BTreeSet;

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn flag(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}
fn number(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite())
}
fn spatial_level(v: &str) -> Option<u8> {
    match v {
        "metro" => Some(0),
        "region" => Some(1),
        "ward" => Some(2),
        "building" => Some(3),
        "room" => Some(4),
        _ => None,
    }
}

#[derive(Default)]
struct Admission {
    disposition: &'static str,
    reasons: BTreeSet<String>,
}
impl Admission {
    fn new() -> Self {
        Self {
            disposition: "Admissible",
            reasons: BTreeSet::new(),
        }
    }
    fn block(&mut self, code: &str) {
        self.reasons.insert(code.to_owned());
        self.disposition = "Blocked";
    }
    fn unknown(&mut self, code: &str) {
        self.reasons.insert(code.to_owned());
        if self.disposition == "Admissible" {
            self.disposition = "Unknown";
        }
    }
    fn unsupported(&mut self, code: &str) {
        self.reasons.insert(code.to_owned());
        self.disposition = "Unsupported";
    }
}

/// Recomputes one claim from input facts only. It never reads the fixture expected output.
pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let level = text(input, "requested_level");
    let source = input.get("source").unwrap_or(&Value::Null);
    let weather = input.get("weather").unwrap_or(&Value::Null);
    let spatial = input.get("spatial").unwrap_or(&Value::Null);
    let population = input.get("population").unwrap_or(&Value::Null);
    let cohort = input.get("cohort").unwrap_or(&Value::Null);
    let model = input.get("model").unwrap_or(&Value::Null);
    let measurement = input.get("measurement").unwrap_or(&Value::Null);
    let intervention = input.get("intervention").unwrap_or(&Value::Null);
    let strategy = input.get("strategy").unwrap_or(&Value::Null);
    let mut out = Admission::new();

    if !matches!(level, "H0" | "H1" | "H2" | "H3" | "H4" | "H5") {
        out.unsupported("CLAIM_LEVEL_UNSUPPORTED");
    }
    if !flag(source, "exists") {
        out.unknown("SOURCE_EXISTENCE_UNVERIFIED");
    }
    if text(source, "rights_state") != "verified" {
        out.block("SOURCE_RIGHTS_NOT_VERIFIED");
    }

    let native = spatial_level(text(spatial, "native_level"));
    let target = spatial_level(text(spatial, "target_level"));
    match (native, target) {
        (Some(n), Some(t)) if t > n => out.block("SPATIAL_PRECISION_PROMOTION"),
        (Some(_), Some(_)) => {}
        _ => out.unsupported("SPATIAL_LEVEL_UNSUPPORTED"),
    }
    if !flag(spatial, "boundary_match") {
        out.unknown("BOUNDARY_VINTAGE_MISMATCH");
    }

    if level != "H0" {
        let kind = text(weather, "class");
        let usage = text(weather, "use");
        if kind.is_empty() || usage.is_empty() {
            out.unknown("WEATHER_CLASS_OR_USE_MISSING");
        }
        if kind == "typical_meteorological_year" && usage != "energy_screening" {
            out.block("TMY_NOT_EXTREME_OR_EVENT_WEATHER");
        }
        if kind == "future_projection"
            && (text(weather, "scenario_id").is_empty()
                || weather.get("period_start").is_none_or(Value::is_null)
                || weather.get("period_end").is_none_or(Value::is_null))
        {
            out.unknown("FUTURE_PROJECTION_SCOPE_INCOMPLETE");
        }
        if kind == "observed_event"
            && (weather.get("period_start").is_none_or(Value::is_null)
                || weather.get("period_end").is_none_or(Value::is_null))
        {
            out.unknown("OBSERVED_EVENT_PERIOD_MISSING");
        }
    }

    if matches!(level, "H2" | "H3" | "H4" | "H5") {
        let kind = text(population, "kind");
        if kind == "ward_estimate" && target.is_some_and(|x| x > 2) {
            out.block("WARD_ESTIMATE_TO_BUILDING_PROMOTION");
        }
        if kind == "survey_sample" {
            if text(population, "claim_mode") == "census" {
                out.block("SURVEY_SAMPLE_PROMOTED_TO_CENSUS");
            }
            if text(population, "claim_mode") != "sample_estimate"
                || !flag(population, "weights_documented")
                || !flag(population, "uncertainty_documented")
            {
                out.unknown("SURVEY_DESIGN_OR_UNCERTAINTY_INCOMPLETE");
            }
        }
        if !matches!(kind, "ward_estimate" | "survey_sample" | "census") {
            out.unknown("POPULATION_SOURCE_KIND_UNKNOWN");
        }
        match number(cohort, "coverage_fraction") {
            Some(c) if !(0.0..=1.0).contains(&c) => {
                out.unsupported("COHORT_COVERAGE_FRACTION_INVALID")
            }
            Some(c) if c < 1.0 && flag(cohort, "require_full_coverage") => {
                out.block("INCOMPLETE_COHORT_COVERAGE")
            }
            Some(c) if c < 1.0 => {
                out.reasons
                    .insert("INCOMPLETE_COHORT_COVERAGE_REPORTED".to_owned());
            }
            Some(_) => {}
            None => out.unknown("COHORT_COVERAGE_UNKNOWN"),
        }
    }

    // H3 needs a model. H4/H5 need measured data and do not require a model.
    if level == "H3" {
        if !flag(model, "required") {
            out.unknown("THERMAL_MODEL_RECEIPT_REQUIRED");
        }
        if text(model, "process_exit") != "success" {
            out.block("THERMAL_MODEL_PROCESS_NOT_SUCCESS");
        }
        if text(model, "convergence") != "converged" {
            out.block("THERMAL_MODEL_NOT_CONVERGED");
        }
        if !flag(model, "scope_match") {
            out.unknown("THERMAL_MODEL_SCOPE_MISMATCH");
        }
        if text(model, "validation_scope") != "in_scope" {
            out.unknown("THERMAL_MODEL_VALIDATION_SCOPE_UNRESOLVED");
        }
        if text(model, "evidence_state") != "current" {
            out.unknown("THERMAL_MODEL_EVIDENCE_NOT_CURRENT");
        }
        let outputs = model.get("outputs").and_then(Value::as_array);
        let has_operative = outputs.is_some_and(|x| {
            x.iter()
                .any(|v| v.as_str() == Some("operative_temperature"))
        });
        let has_overheating = outputs.is_some_and(|x| {
            x.iter()
                .any(|v| v.as_str() == Some("overheating_degree_hours"))
        });
        if !has_operative || !has_overheating {
            out.block("THERMAL_SERVICE_OUTPUTS_INCOMPLETE");
        }
        if flag(strategy, "uses_natural_ventilation") && !flag(strategy, "ventilation_available") {
            out.block("VENTILATION_UNAVAILABLE_IN_SCENARIO");
        }
    }

    if matches!(level, "H4" | "H5") {
        if !flag(measurement, "required") {
            out.unknown("MEASURED_DATA_RECEIPT_REQUIRED");
        }
        if text(measurement, "source_kind") != "measured" {
            out.block("MODELLED_DATA_CLAIMED_AS_MEASURED");
        }
        if text(measurement, "calibration_state") != "current" {
            out.unknown("SENSOR_CALIBRATION_NOT_CURRENT");
        }
        if text(measurement, "quality_state") != "valid" {
            out.unknown("SENSOR_DATA_QUALITY_NOT_VALID");
        }
        if text(measurement, "consent_state") != "approved"
            || !flag(measurement, "spatial_aggregation_allowed")
        {
            out.block("MEASUREMENT_PRIVACY_OR_CONSENT_GATE_FAILED");
        }
    }

    if level == "H5" {
        if text(intervention, "status") == "predicted" {
            out.block("PREDICTED_INTERVENTION_CLAIMED_AS_EFFECT");
        }
        if text(intervention, "status") != "accepted" {
            out.unknown("INTERVENTION_NOT_ACCEPTED");
        }
        if !flag(intervention, "installation_receipt")
            || !flag(intervention, "inspection_receipt")
            || !flag(intervention, "commissioning_receipt")
        {
            out.unknown("INSTALLATION_INSPECTION_OR_COMMISSIONING_EVIDENCE_MISSING");
        }
        if !flag(intervention, "pre_post_measurements") || !flag(intervention, "comparison_design")
        {
            out.unknown("PRE_POST_OR_COMPARISON_EVIDENCE_MISSING");
        }
    }

    let unknown_fraction = number(cohort, "coverage_fraction")
        .filter(|x| (0.0..1.0).contains(x))
        .map(|x| 1.0 - x);
    json!({
        "case_id": case_id,
        "disposition": out.disposition,
        "claim_level": level,
        "allowed": out.disposition == "Admissible",
        "reason_codes": out.reasons.into_iter().collect::<Vec<_>>(),
        "reported_unknown_cohort_fraction": unknown_fraction
    })
}

fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            x.as_f64().zip(y.as_f64()).is_some_and(|(x, y)| x == y)
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && y.iter()
                    .all(|(k, v)| x.get(k).is_some_and(|a| equivalent(a, v)))
        }
        _ => a == b,
    }
}
pub fn matches_expected(actual: &Value, expected: &Value) -> bool {
    expected.as_object().is_some_and(|fields| {
        fields
            .iter()
            .all(|(k, v)| actual.get(k).is_some_and(|a| equivalent(a, v)))
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_case, matches_expected};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn corpus() -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/city-heat-retrofit/CITY_HEAT_EVIDENCE_CORPUS_V0_1.json");
        let bytes = std::fs::read(path).expect("heat evidence corpus must exist");
        serde_json::from_slice(&bytes).expect("heat evidence corpus must parse")
    }
    fn case(id: &str) -> Value {
        corpus()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["case_id"] == id)
            .unwrap()
            .clone()
    }
    #[test]
    fn corpus_has_27_unique_synthetic_cases() {
        let c = corpus();
        assert_eq!(c["classification"], "synthetic_fixture_only");
        assert_eq!(c["corpus_id"], "CITY-HEAT-EVIDENCE-CORPUS-V0.1");
        let xs = c["cases"].as_array().unwrap();
        assert_eq!(xs.len(), 27);
        let ids = xs
            .iter()
            .map(|x| x["case_id"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), xs.len());
    }
    #[test]
    fn oracle_matches_every_known_answer() {
        for x in corpus()["cases"].as_array().unwrap() {
            let actual = evaluate_case(x);
            assert!(
                matches_expected(&actual, &x["expected"]),
                "{} mismatch
actual: {}
expected: {}",
                x["case_id"],
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&x["expected"]).unwrap()
            );
        }
    }
    #[test]
    fn programme_impact_claim_is_reserved_and_cannot_pass_as_h5() {
        let result = evaluate_case(&case("HA-027"));
        assert_eq!(result["disposition"], "Unsupported");
        assert_eq!(result["claim_level"], "H6");
        assert_eq!(result["allowed"], false);
        assert_eq!(
            result["reason_codes"],
            serde_json::json!(["CLAIM_LEVEL_UNSUPPORTED"])
        );
    }

    #[test]
    fn ward_and_sample_evidence_cannot_be_promoted() {
        for id in ["HA-007", "HA-008"] {
            assert_eq!(evaluate_case(&case(id))["disposition"], "Blocked");
        }
    }
    #[test]
    fn tmy_is_limited_to_typical_weather_screening() {
        assert_eq!(evaluate_case(&case("HA-002"))["disposition"], "Admissible");
        assert_eq!(evaluate_case(&case("HA-003"))["disposition"], "Blocked");
    }
    #[test]
    fn measured_claims_do_not_require_a_model() {
        assert_eq!(evaluate_case(&case("HA-018"))["disposition"], "Admissible");
        assert_eq!(evaluate_case(&case("HA-021"))["disposition"], "Admissible");
    }
    #[test]
    fn stale_calibration_and_modelled_to_measured_promotion_fail() {
        assert_eq!(evaluate_case(&case("HA-019"))["disposition"], "Unknown");
        assert_eq!(evaluate_case(&case("HA-020"))["disposition"], "Blocked");
    }
    #[test]
    fn model_scope_and_convergence_are_mandatory_for_h3() {
        for id in ["HA-014", "HA-015", "HA-016", "HA-017"] {
            assert_ne!(evaluate_case(&case(id))["disposition"], "Admissible");
        }
    }
    #[test]
    fn incomplete_cohort_coverage_is_reported_or_blocks_full_coverage() {
        let x = evaluate_case(&case("HA-011"));
        assert_eq!(x["disposition"], "Admissible");
        let fraction = x["reported_unknown_cohort_fraction"].as_f64().unwrap();
        assert!((fraction - 0.3).abs() < 1e-12);
        assert_eq!(evaluate_case(&case("HA-012"))["disposition"], "Blocked");
    }
}
