//! Synthetic reference oracle for time-dependent service continuity.
//! Consumes owner-native access-state receipts; does not recompute routes,
//! service capacity, productive closure, or hazard probabilities.

use serde_json::{Value, json};
use std::collections::BTreeSet;

const CURRENT: &str = "synthetic_current";
const OWNER: &str = "sol-atlas";
const CONTRACT: &str = "CITY-MORPH-ACCESS-001";

fn array<'a>(v: &'a Value, k: &str) -> Option<&'a [Value]> {
    v.get(k).and_then(Value::as_array).map(Vec::as_slice)
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn tick(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_i64)
}
fn unsupported(case_id: &str, reasons: BTreeSet<String>) -> Value {
    json!({
        "case_id": case_id,
        "disposition": "Unsupported",
        "reason_codes": reasons.into_iter().collect::<Vec<_>>(),
        "requirements": []
    })
}

#[derive(Default)]
struct Summary {
    adequate: i64,
    constrained: i64,
    inaccessible: i64,
    unknown: i64,
    unknown_after_degradation: i64,
    max_degraded: i64,
    degraded_streak: i64,
    first_degradation: Option<i64>,
    last_degradation_end: Option<i64>,
    unknown_since_last_degradation: bool,
    adequate_run_start: Option<i64>,
    adequate_run: i64,
    recovered_start: Option<i64>,
    recovery_confirmed: Option<i64>,
}
impl Summary {
    fn consume(&mut self, state: &str, start: i64, end: i64, confirm: i64) {
        let duration = end - start;
        match state {
            "Adequate" => {
                self.adequate += duration;
                self.degraded_streak = 0;
                if self.adequate_run == 0 {
                    self.adequate_run_start = Some(start);
                }
                self.adequate_run += duration;
                if self.first_degradation.is_some()
                    && self.recovered_start.is_none()
                    && self.adequate_run >= confirm
                {
                    self.recovered_start = self.adequate_run_start;
                    self.recovery_confirmed = self.adequate_run_start.map(|s| s + confirm);
                }
            }
            "Constrained" => {
                self.constrained += duration;
                self.degrade(start, duration);
            }
            "Inaccessible" => {
                self.inaccessible += duration;
                self.degrade(start, duration);
            }
            _ => {
                self.unknown += duration;
                if self.first_degradation.is_some() {
                    self.unknown_after_degradation += duration;
                }
                if self.last_degradation_end.is_some() {
                    self.unknown_since_last_degradation = true;
                }
                self.degraded_streak = 0;
                self.adequate_run = 0;
                self.adequate_run_start = None;
            }
        }
    }
    fn degrade(&mut self, start: i64, duration: i64) {
        self.first_degradation.get_or_insert(start);
        self.last_degradation_end = Some(start + duration);
        self.unknown_since_last_degradation = false;
        self.degraded_streak += duration;
        self.max_degraded = self.max_degraded.max(self.degraded_streak);
        self.adequate_run = 0;
        self.adequate_run_start = None;
    }
    fn finish(self, req: &Value, start: i64, end: i64) -> Value {
        let state = if self.first_degradation.is_none() {
            "NoKnownInterruption"
        } else if self.recovered_start.is_some() {
            "Recovered"
        } else if self.unknown_after_degradation > 0 {
            "RecoveryUnknown"
        } else {
            "NotRecovered"
        };
        let recovery_timing_precision = if self.first_degradation.is_none() {
            "not_applicable"
        } else if self.recovered_start.is_some() {
            if self.unknown_since_last_degradation {
                "bounded"
            } else {
                "exact"
            }
        } else if state == "RecoveryUnknown" {
            "unknown"
        } else {
            "unresolved"
        };
        let recovery_start_lower_exclusive =
            if self.recovered_start.is_some() || state == "RecoveryUnknown" {
                self.last_degradation_end
            } else {
                None
            };
        let recovery_start_upper_inclusive = self.recovered_start;
        let recovery_time_lower_exclusive = self
            .first_degradation
            .zip(recovery_start_lower_exclusive)
            .map(|(first, lower)| lower - first);
        let recovery_time_upper_inclusive = self
            .first_degradation
            .zip(recovery_start_upper_inclusive)
            .map(|(first, upper)| upper - first);
        let recovery_time = if recovery_timing_precision == "exact" {
            recovery_time_upper_inclusive
        } else {
            None
        };
        json!({
            "requirement_id": text(req, "requirement_id"),
            "group_id": text(req, "group_id"),
            "service_category": text(req, "service_category"),
            "horizon_start_tick": start,
            "horizon_end_tick": end,
            "total_ticks": end - start,
            "adequate_ticks": self.adequate,
            "constrained_ticks": self.constrained,
            "inaccessible_ticks": self.inaccessible,
            "unknown_ticks": self.unknown,
            "unknown_after_first_degradation_ticks": self.unknown_after_degradation,
            "max_contiguous_degraded_ticks": self.max_degraded,
            "first_degradation_tick": self.first_degradation,
            "last_known_degradation_end_tick": self.last_degradation_end,
            "recovery_state": state,
            "recovered_start_tick": self.recovered_start,
            "recovery_start_lower_exclusive_tick": recovery_start_lower_exclusive,
            "recovery_start_upper_inclusive_tick": recovery_start_upper_inclusive,
            "recovery_timing_precision": recovery_timing_precision,
            "recovery_confirmed_at_tick": self.recovery_confirmed,
            "recovery_time_ticks": recovery_time,
            "recovery_time_lower_exclusive_ticks": recovery_time_lower_exclusive,
            "recovery_time_upper_inclusive_ticks": recovery_time_upper_inclusive
        })
    }
}

fn admitted_state(
    case_id: &str,
    input: &Value,
    phase_id: &str,
    req_id: &str,
    r: &Value,
) -> &'static str {
    let expected = format!("synthetic-subject:{case_id}:{phase_id}:{req_id}");
    if text(input, "evidence_state") != CURRENT
        || text(r, "evidence_state") != CURRENT
        || text(r, "source_owner") != OWNER
        || text(r, "source_contract") != CONTRACT
        || text(r, "source_revision") != text(input, "expected_source_revision")
        || text(r, "source_scope_digest") != text(input, "scope_digest")
        || text(r, "source_receipt_id").is_empty()
        || text(r, "scenario_id") != case_id
        || text(r, "source_subject_digest") != expected
    {
        return "Unknown";
    }
    match text(r, "outcome") {
        "Adequate" => "Adequate",
        "Constrained" => "Constrained",
        "Inaccessible" => "Inaccessible",
        _ => "Unknown",
    }
}

pub fn evaluate_case(case: &Value) -> Value {
    let case_id = text(case, "case_id");
    let input = case.get("input").unwrap_or(&Value::Null);
    let mut errors = BTreeSet::new();
    if text(input, "scope_digest").is_empty() {
        errors.insert("TIMELINE_SCOPE_DIGEST_MISSING".to_owned());
    }
    if text(input, "expected_source_revision").is_empty() {
        errors.insert("EXPECTED_OWNER_SOURCE_REVISION_MISSING".to_owned());
    }
    let Some(confirm) = tick(input, "recovery_confirmation_ticks") else {
        errors.insert("RECOVERY_CONFIRMATION_POLICY_INVALID".to_owned());
        return unsupported(case_id, errors);
    };
    if confirm < 1 {
        errors.insert("RECOVERY_CONFIRMATION_POLICY_INVALID".to_owned());
    }
    let Some(reqs) = array(input, "requirements") else {
        errors.insert("REQUIREMENTS_MISSING".to_owned());
        return unsupported(case_id, errors);
    };
    let Some(phases) = array(input, "phases") else {
        errors.insert("PHASES_MISSING".to_owned());
        return unsupported(case_id, errors);
    };
    if reqs.is_empty() {
        errors.insert("REQUIREMENTS_EMPTY".to_owned());
    }
    if phases.is_empty() {
        errors.insert("PHASES_EMPTY".to_owned());
    }

    let mut req_ids = BTreeSet::new();
    for req in reqs {
        let id = text(req, "requirement_id");
        if id.is_empty() || !req_ids.insert(id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_REQUIREMENT_ID".to_owned());
        }
        if text(req, "group_id").is_empty() || text(req, "service_category").is_empty() {
            errors.insert("REQUIREMENT_SEMANTICS_MISSING".to_owned());
        }
    }

    let mut phase_ids = BTreeSet::new();
    let mut all_source_receipt_ids = BTreeSet::new();
    let mut ordered = Vec::<(&Value, i64, i64)>::new();
    for phase in phases {
        let id = text(phase, "phase_id");
        if id.is_empty() || !phase_ids.insert(id.to_owned()) {
            errors.insert("DUPLICATE_OR_MISSING_PHASE_ID".to_owned());
        }
        let (Some(start), Some(end)) = (tick(phase, "start_tick"), tick(phase, "end_tick")) else {
            errors.insert("PHASE_INTERVAL_INVALID".to_owned());
            continue;
        };
        if start < 0 || end <= start {
            errors.insert("PHASE_INTERVAL_INVALID".to_owned());
            continue;
        }
        ordered.push((phase, start, end));

        let Some(events) = array(phase, "event_ids") else {
            errors.insert("PHASE_EVENT_IDS_MISSING".to_owned());
            continue;
        };
        let mut event_ids = BTreeSet::new();
        for event in events {
            let Some(event) = event.as_str() else {
                errors.insert("PHASE_EVENT_ID_INVALID".to_owned());
                continue;
            };
            if event.is_empty() || !event_ids.insert(event.to_owned()) {
                errors.insert("DUPLICATE_OR_MISSING_PHASE_EVENT_ID".to_owned());
            }
        }

        let Some(receipts) = array(phase, "access_receipts") else {
            errors.insert("PHASE_ACCESS_RECEIPTS_MISSING".to_owned());
            continue;
        };
        let mut seen = BTreeSet::new();
        for receipt in receipts {
            let req_id = text(receipt, "requirement_id");
            if req_id.is_empty() || !seen.insert(req_id.to_owned()) {
                errors.insert("DUPLICATE_PHASE_REQUIREMENT_RECEIPT".to_owned());
            }
            if !req_ids.contains(req_id) {
                errors.insert("PHASE_REQUIREMENT_RECEIPT_UNEXPECTED".to_owned());
            }
            let receipt_id = text(receipt, "source_receipt_id");
            if !receipt_id.is_empty() && !all_source_receipt_ids.insert(receipt_id.to_owned()) {
                errors.insert("DUPLICATE_SOURCE_RECEIPT_ID".to_owned());
            }
            if !matches!(
                text(receipt, "outcome"),
                "Adequate" | "Constrained" | "Inaccessible" | "Unknown"
            ) {
                errors.insert("ACCESS_OUTCOME_UNSUPPORTED".to_owned());
            }
        }
        for req_id in &req_ids {
            if !seen.contains(req_id) {
                errors.insert("PHASE_REQUIREMENT_RECEIPT_MISSING".to_owned());
            }
        }
    }
    ordered.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| text(a.0, "phase_id").cmp(text(b.0, "phase_id")))
    });
    if let Some((_, first, _)) = ordered.first()
        && *first != 0
    {
        errors.insert("TIMELINE_MUST_START_AT_ZERO".to_owned());
    }
    for pair in ordered.windows(2) {
        if pair[1].1 < pair[0].2 {
            errors.insert("PHASE_INTERVAL_OVERLAP".to_owned());
        } else if pair[1].1 > pair[0].2 {
            errors.insert("PHASE_TIMELINE_GAP".to_owned());
        }
    }
    if !errors.is_empty() {
        return unsupported(case_id, errors);
    }

    let horizon_start = ordered.first().map(|p| p.1).unwrap_or(0);
    let horizon_end = ordered.last().map(|p| p.2).unwrap_or(0);
    let mut reqs_sorted = reqs.iter().collect::<Vec<_>>();
    reqs_sorted.sort_by(|a, b| text(a, "requirement_id").cmp(text(b, "requirement_id")));
    let mut outputs = Vec::new();

    for req in reqs_sorted {
        let req_id = text(req, "requirement_id");
        let mut summary = Summary::default();
        for (phase, start, end) in &ordered {
            let phase_id = text(phase, "phase_id");
            let receipt = array(phase, "access_receipts")
                .unwrap()
                .iter()
                .find(|r| text(r, "requirement_id") == req_id)
                .unwrap();
            let state = admitted_state(case_id, input, phase_id, req_id, receipt);
            summary.consume(state, *start, *end, confirm);
        }
        outputs.push(summary.finish(req, horizon_start, horizon_end));
    }
    json!({
        "case_id": case_id,
        "disposition": "Evaluated",
        "reason_codes": [],
        "requirements": outputs
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
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/research/city-morph-continuity/CITY_MORPH_CONTINUITY_CORPUS_V0_1.json",
        );
        let bytes = std::fs::read(path).expect("continuity corpus must exist");
        serde_json::from_slice(&bytes).expect("continuity corpus must parse")
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
    fn corpus_has_30_unique_synthetic_cases() {
        let c = corpus();
        assert_eq!(c["classification"], "synthetic_fixture_only");
        assert_eq!(c["corpus_id"], "CITY-MORPH-CONTINUITY-CORPUS-V0.1");
        let xs = c["cases"].as_array().unwrap();
        assert_eq!(xs.len(), 30);
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
                "{} mismatch\nactual: {}\nexpected: {}",
                x["case_id"],
                serde_json::to_string_pretty(&actual).unwrap(),
                serde_json::to_string_pretty(&x["expected"]).unwrap()
            );
        }
    }

    #[test]
    fn stale_and_mismatched_receipts_are_unknown_not_adequate() {
        for id in ["CR-008", "CR-009", "CR-023", "CR-024", "CR-025", "CR-026"] {
            let r = evaluate_case(&case(id));
            assert_eq!(r["requirements"][0]["adequate_ticks"], 0);
            assert_eq!(
                r["requirements"][0]["unknown_ticks"],
                if id == "CR-025" || id == "CR-026" {
                    5
                } else {
                    4
                }
            );
        }
    }

    #[test]
    fn recovery_requires_the_declared_confirmation_duration() {
        assert_eq!(
            evaluate_case(&case("CR-003"))["requirements"][0]["recovery_state"],
            "NotRecovered"
        );
        let r = evaluate_case(&case("CR-002"));
        assert_eq!(r["requirements"][0]["recovery_state"], "Recovered");
        assert_eq!(r["requirements"][0]["recovery_time_ticks"], 2);
    }

    #[test]
    fn recovery_time_is_bounded_when_evidence_gap_precedes_confirmed_adequacy() {
        let result = evaluate_case(&case("CR-030"));
        let requirement = &result["requirements"][0];
        assert_eq!(requirement["recovery_state"], "Recovered");
        assert_eq!(requirement["recovery_timing_precision"], "bounded");
        assert_eq!(requirement["last_known_degradation_end_tick"], 5);
        assert_eq!(requirement["recovered_start_tick"], 7);
        assert_eq!(requirement["recovery_start_lower_exclusive_tick"], 5);
        assert_eq!(requirement["recovery_start_upper_inclusive_tick"], 7);
        assert_eq!(requirement["recovery_time_ticks"], Value::Null);
        assert_eq!(requirement["recovery_time_lower_exclusive_ticks"], 2);
        assert_eq!(requirement["recovery_time_upper_inclusive_ticks"], 4);
        assert_eq!(requirement["recovery_confirmed_at_tick"], 9);

        let exact = evaluate_case(&case("CR-002"));
        let exact_requirement = &exact["requirements"][0];
        assert_eq!(exact_requirement["recovery_timing_precision"], "exact");
        assert_eq!(exact_requirement["recovery_time_ticks"], 2);
        assert_eq!(exact_requirement["recovery_time_lower_exclusive_ticks"], 2);
        assert_eq!(exact_requirement["recovery_time_upper_inclusive_ticks"], 2);
    }

    #[test]
    fn pre_outage_unknown_does_not_poison_recovery_status() {
        let before = evaluate_case(&case("CR-027"));
        assert_eq!(before["requirements"][0]["recovery_state"], "NotRecovered");
        assert_eq!(
            before["requirements"][0]["unknown_after_first_degradation_ticks"],
            0
        );
        let after = evaluate_case(&case("CR-020"));
        assert_eq!(
            after["requirements"][0]["recovery_state"],
            "RecoveryUnknown"
        );
        assert_eq!(
            after["requirements"][0]["unknown_after_first_degradation_ticks"],
            3
        );
        assert_eq!(
            after["requirements"][0]["recovery_timing_precision"],
            "unknown"
        );
        assert_eq!(
            after["requirements"][0]["recovery_start_lower_exclusive_tick"],
            4
        );
        assert_eq!(
            after["requirements"][0]["recovery_start_upper_inclusive_tick"],
            Value::Null
        );
    }

    #[test]
    fn adjacent_degraded_phases_form_one_streak() {
        assert_eq!(
            evaluate_case(&case("CR-005"))["requirements"][0]["max_contiguous_degraded_ticks"],
            5
        );
    }

    #[test]
    fn unknown_duration_is_not_counted_as_adequate_or_degraded() {
        let r = evaluate_case(&case("CR-004"));
        assert_eq!(r["requirements"][0]["adequate_ticks"], 5);
        assert_eq!(r["requirements"][0]["unknown_ticks"], 2);
        assert_eq!(r["requirements"][0]["constrained_ticks"], 0);
    }

    #[test]
    fn phase_permutation_does_not_change_result() {
        let mut x = case("CR-017");
        let before = evaluate_case(&x);
        x["input"]["phases"].as_array_mut().unwrap().reverse();
        assert_eq!(before, evaluate_case(&x));
    }

    #[test]
    fn malformed_timeline_structures_fail_closed() {
        for id in [
            "CR-010", "CR-011", "CR-012", "CR-013", "CR-014", "CR-015", "CR-016", "CR-028",
            "CR-029",
        ] {
            assert_eq!(evaluate_case(&case(id))["disposition"], "Unsupported");
        }
    }

    #[test]
    fn group_requirements_remain_separate_during_common_cause_events() {
        let r = evaluate_case(&case("CR-007"));
        assert_eq!(r["requirements"][0]["inaccessible_ticks"], 3);
        assert_eq!(r["requirements"][1]["adequate_ticks"], 8);
        let r = evaluate_case(&case("CR-018"));
        assert_eq!(r["requirements"][0]["inaccessible_ticks"], 4);
        assert_eq!(r["requirements"][1]["inaccessible_ticks"], 4);
    }
}
