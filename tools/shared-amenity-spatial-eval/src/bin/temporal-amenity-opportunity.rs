#[rustfmt::skip]
mod implementation {
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs, process};

const FIXTURE_PATH: &str = "tools/shared-amenity-spatial-eval/fixtures/TEMPORAL_AMENITY_OPPORTUNITY_FIXTURES_V0.1.json";
const EXPECTED_SCHEMA: &str = "luminous.temporal-amenity-opportunity-fixtures";
const EXPECTED_VERSION: &str = "0.1.0";
const EXPECTED_MATURITY: &str = "SYNTHETIC_TEMPORAL_JOIN_ORACLE_ONLY";
const MAX_ARRIVAL_CANDIDATES: i64 = 1_000;
const SOL_ATLAS_SCOPE: &str = "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/623";
const SOL_ATLAS_ACCESS: &str = "https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555";
const MYCELIX_SERVICE: &str = "https://github.com/Luminous-Dynamics/mycelix/issues/6328";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSet {
    schema: String,
    schema_version: String,
    maturity: String,
    claim_ceiling: String,
    owner_bindings: OwnerBindings,
    time_basis_contract: TimeBasisContract,
    cases: Vec<OpportunityCase>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerBindings { scope: String, access: String, service: String }
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeBasisContract {
    tick_seconds: u64,
    origin: String,
    rule: String,
    interval_semantics: String,
    uncertainty_semantics: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OpportunityCase {
    opportunity_id: String,
    scope: EvaluationScope,
    access: AccessReceipt,
    calendar: ServiceCalendar,
    required_service_duration_ticks: i64,
    expected: ExpectedResult,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationScope {
    scope_digest: String,
    service_id: String,
    functional_unit_id: String,
    period_id: String,
    time_basis_id: String,
    horizon_start_tick: i64,
    horizon_end_tick: i64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessReceipt {
    owner: String,
    owner_reference: String,
    owner_source_id: String,
    owner_revision: String,
    scope_digest: String,
    time_basis_id: String,
    access_state: String,
    evidence_state: String,
    arrival_earliest_tick: i64,
    arrival_latest_tick: i64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceCalendar {
    owner: String,
    owner_reference: String,
    owner_source_id: String,
    owner_revision: String,
    scope_digest: String,
    time_basis_id: String,
    coverage_state: String,
    evidence_state: String,
    service_windows: Vec<ServiceWindow>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceWindow {
    window_id: String,
    owner: String,
    owner_reference: String,
    owner_source_id: String,
    owner_revision: String,
    scope_digest: String,
    time_basis_id: String,
    start_tick: i64,
    end_tick: i64,
    operational_state: String,
    evidence_state: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
struct ExpectedResult {
    disposition: String,
    reason_code: String,
    possible_arrival_count: u64,
    available_arrival_count: u64,
    unavailable_arrival_count: u64,
    unknown_arrival_count: u64,
}
#[derive(Debug, Serialize)]
struct OpportunityResult {
    opportunity_id: String,
    scope_digest: String,
    service_id: String,
    functional_unit_id: String,
    period_id: String,
    time_basis_id: String,
    disposition: String,
    reason_code: String,
    access_state: String,
    calendar_coverage_state: String,
    arrival_earliest_tick: i64,
    arrival_latest_tick: i64,
    required_service_duration_ticks: i64,
    service_finish_latest_tick: i64,
    possible_arrival_count: u64,
    available_arrival_count: u64,
    unavailable_arrival_count: u64,
    unknown_arrival_count: u64,
    service_owner_revision: String,
    service_calendar_evidence_state: String,
    access_owner_revision: String,
    access_evidence_state: String,
}
#[derive(Debug, Serialize)]
struct EvaluationOutput {
    schema: String,
    schema_version: String,
    maturity: String,
    claim_ceiling: String,
    fixture_sha256: String,
    time_basis_tick_seconds: u64,
    time_basis_origin: String,
    interval_semantics: String,
    opportunity_count: usize,
    disposition_counts: DispositionCounts,
    results: Vec<OpportunityResult>,
}
#[derive(Debug, Serialize)]
struct DispositionCounts {
    available: usize,
    unavailable: usize,
    unknown: usize,
    not_comparable: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotState { Available, Unavailable, Unknown }

// Reject duplicate keys recursively before values enter serde_json::Map.
struct UniqueValue(Value);
impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}
struct UniqueValueVisitor;
impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a JSON value with unique object keys")
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::Bool(value))) }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::from(value))) }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::from(value))) }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where E: de::Error {
        let n = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("non-finite JSON number"))?;
        Ok(UniqueValue(Value::Number(n)))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::String(value.to_owned()))) }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::String(value))) }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::Null)) }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where E: de::Error { Ok(UniqueValue(Value::Null)) }
    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where D: Deserializer<'de> { UniqueValue::deserialize(d) }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut values = Vec::new();
        while let Some(UniqueValue(value)) = seq.next_element::<UniqueValue>()? {
            values.push(value);
        }
        Ok(UniqueValue(Value::Array(values)))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de> {
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

fn parse_fixture(bytes: &[u8]) -> Result<FixtureSet, String> {
    let UniqueValue(value) = serde_json::from_slice(bytes)
        .map_err(|e| format!("invalid or ambiguous JSON: {e}"))?;
    serde_json::from_value(value).map_err(|e| format!("fixture schema error: {e}"))
}

fn validate_fixture(f: &FixtureSet) -> Result<(), String> {
    if f.schema != EXPECTED_SCHEMA || f.schema_version != EXPECTED_VERSION || f.maturity != EXPECTED_MATURITY {
        return Err("fixture identity, version or maturity changed".to_owned());
    }
    if f.claim_ceiling.trim().is_empty() { return Err("claim ceiling must be explicit".to_owned()); }
    if f.owner_bindings.scope != SOL_ATLAS_SCOPE
        || f.owner_bindings.access != SOL_ATLAS_ACCESS
        || f.owner_bindings.service != MYCELIX_SERVICE {
        return Err("canonical owner bindings changed".to_owned());
    }
    let t = &f.time_basis_contract;
    if t.tick_seconds != 900
        || t.origin != "SYNTHETIC_LOCAL_DAY_START"
        || t.interval_semantics != "HALF_OPEN_START_INCLUSIVE_END_EXCLUSIVE"
        || t.rule != "All tick values are already normalized to this exact time basis by owner systems; this tool performs no timezone conversion or calendar inference."
        || t.uncertainty_semantics != "arrival_earliest_tick and arrival_latest_tick are inclusive possible arrival ticks" {
        return Err("time-basis contract is incomplete or differs from the frozen 900-second tick profile".to_owned());
    }
    if f.cases.is_empty() { return Err("fixture set must contain at least one opportunity".to_owned()); }
    let mut ids = BTreeSet::new();
    for c in &f.cases {
        validate_case(c)?;
        if !ids.insert(c.opportunity_id.as_str()) {
            return Err(format!("duplicate opportunity identifier: {}", c.opportunity_id));
        }
    }
    Ok(())
}

fn validate_case(c: &OpportunityCase) -> Result<(), String> {
    let id = &c.opportunity_id;
    if id.trim().is_empty() || c.scope.scope_digest.trim().is_empty()
        || c.scope.service_id.trim().is_empty() || c.scope.functional_unit_id.trim().is_empty()
        || c.scope.period_id.trim().is_empty() || c.scope.time_basis_id.trim().is_empty() {
        return Err(format!("{id}: opportunity and scope identities must be non-empty"));
    }
    if c.scope.horizon_start_tick < 0 || c.scope.horizon_end_tick <= c.scope.horizon_start_tick {
        return Err(format!("{id}: invalid evaluation horizon"));
    }
    if c.required_service_duration_ticks <= 0 {
        return Err(format!("{id}: required service duration must be positive"));
    }
    if c.access.arrival_earliest_tick < c.scope.horizon_start_tick
        || c.access.arrival_latest_tick < c.access.arrival_earliest_tick
        || c.access.arrival_latest_tick >= c.scope.horizon_end_tick {
        return Err(format!("{id}: invalid or out-of-horizon arrival interval"));
    }
    let width = c.access.arrival_latest_tick.checked_sub(c.access.arrival_earliest_tick)
        .and_then(|v| v.checked_add(1)).ok_or_else(|| format!("{id}: arrival count overflow"))?;
    if width > MAX_ARRIVAL_CANDIDATES {
        return Err(format!("{id}: arrival uncertainty exceeds {MAX_ARRIVAL_CANDIDATES} candidate ticks"));
    }
    let finish = c.access.arrival_latest_tick.checked_add(c.required_service_duration_ticks)
        .ok_or_else(|| format!("{id}: requested service end overflow"))?;
    if finish > c.scope.horizon_end_tick {
        return Err(format!("{id}: requested service interval exceeds scope horizon"));
    }
    if !["QUALIFIED", "UNKNOWN", "BLOCKED"].contains(&c.access.access_state.as_str()) {
        return Err(format!("{id}: unsupported access state"));
    }
    if !["CURRENT", "STALE", "UNKNOWN"].contains(&c.access.evidence_state.as_str()) {
        return Err(format!("{id}: unsupported access evidence state"));
    }
    if !["COMPLETE", "INCOMPLETE", "UNKNOWN"].contains(&c.calendar.coverage_state.as_str()) {
        return Err(format!("{id}: unsupported calendar coverage state"));
    }
    if !["CURRENT", "STALE", "UNKNOWN"].contains(&c.calendar.evidence_state.as_str()) {
        return Err(format!("{id}: unsupported calendar evidence state"));
    }

    if c.access.owner_source_id.trim().is_empty() || c.access.owner_revision.trim().is_empty()
        || c.calendar.owner_source_id.trim().is_empty() || c.calendar.owner_revision.trim().is_empty() {
        return Err(format!("{id}: owner source identity and revision are required"));
    }
    let mut window_ids = BTreeSet::new();
    let mut ordered = c.calendar.service_windows.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.start_tick
            .cmp(&right.start_tick)
            .then(left.end_tick.cmp(&right.end_tick))
            .then(left.window_id.cmp(&right.window_id))
    });
    for w in &ordered {
        if w.window_id.trim().is_empty() || w.owner_source_id.trim().is_empty() || w.owner_revision.trim().is_empty() {
            return Err(format!("{id}: service-window identity/revision is incomplete"));
        }
        if !window_ids.insert(w.window_id.as_str()) {
            return Err(format!("{id}: duplicate service-window identity"));
        }
        if w.start_tick < c.scope.horizon_start_tick || w.end_tick <= w.start_tick || w.end_tick > c.scope.horizon_end_tick {
            return Err(format!("{id}: invalid or out-of-horizon service window"));
        }
        if !["OPERATIONAL", "CLOSED_FOR_MAINTENANCE", "NOT_COMMISSIONED"].contains(&w.operational_state.as_str()) {
            return Err(format!("{id}: unsupported owner operational state"));
        }
        if !["CURRENT", "STALE", "UNKNOWN"].contains(&w.evidence_state.as_str()) {
            return Err(format!("{id}: unsupported service-window evidence state"));
        }
    }
    for pair in ordered.windows(2) {
        if pair[1].start_tick < pair[0].end_tick {
            return Err(format!("{id}: overlapping or contradictory service windows"));
        }
    }
    Ok(())
}

fn comparable(c: &OpportunityCase) -> Result<(), &'static str> {
    if c.access.scope_digest != c.scope.scope_digest
        || c.calendar.scope_digest != c.scope.scope_digest
        || c.calendar.service_windows.iter().any(|w| w.scope_digest != c.scope.scope_digest) {
        return Err("SCOPE_DIGEST_MISMATCH");
    }
    if c.access.time_basis_id != c.scope.time_basis_id
        || c.calendar.time_basis_id != c.scope.time_basis_id
        || c.calendar.service_windows.iter().any(|w| w.time_basis_id != c.scope.time_basis_id) {
        return Err("TIME_BASIS_MISMATCH");
    }
    if c.access.owner != "Sol-Atlas" || c.access.owner_reference != SOL_ATLAS_ACCESS
        || c.calendar.owner != "Mycelix" || c.calendar.owner_reference != MYCELIX_SERVICE
        || c.calendar.service_windows.iter().any(|w| w.owner != "Mycelix" || w.owner_reference != MYCELIX_SERVICE) {
        return Err("OWNER_CONTRACT_MISMATCH");
    }
    if c.calendar.service_windows.iter().any(|w|
        w.owner_source_id != c.calendar.owner_source_id || w.owner_revision != c.calendar.owner_revision) {
        return Err("MIXED_SERVICE_OWNER_REVISIONS");
    }
    Ok(())
}

fn evaluate_slot(c: &OpportunityCase, arrival: i64) -> Result<SlotState, String> {
    if c.calendar.evidence_state != "CURRENT" || c.calendar.coverage_state == "UNKNOWN" {
        return Ok(SlotState::Unknown);
    }
    let finish = arrival.checked_add(c.required_service_duration_ticks)
        .ok_or_else(|| "requested service interval overflow".to_owned())?;
    let mut cursor = arrival;
    let mut windows = c.calendar.service_windows.iter().collect::<Vec<_>>();
    windows.sort_by(|left, right| {
        left.start_tick
            .cmp(&right.start_tick)
            .then(left.end_tick.cmp(&right.end_tick))
    });
    while cursor < finish {
        let next = windows.iter().find(|w| w.start_tick <= cursor && cursor < w.end_tick);
        let Some(w) = next else {
            return Ok(if c.calendar.coverage_state == "COMPLETE" { SlotState::Unavailable } else { SlotState::Unknown });
        };
        if w.evidence_state != "CURRENT" { return Ok(SlotState::Unknown); }
        if w.operational_state != "OPERATIONAL" { return Ok(SlotState::Unavailable); }
        cursor = finish.min(w.end_tick);
    }
    Ok(SlotState::Available)
}

fn evaluate_case(c: &OpportunityCase) -> Result<OpportunityResult, String> {
    let finish = c.access.arrival_latest_tick.checked_add(c.required_service_duration_ticks)
        .ok_or_else(|| "requested service end overflow".to_owned())?;
    let (disposition, reason, possible, available, unavailable, unknown) = match comparable(c) {
        Err(reason) => ("NOT_COMPARABLE".to_owned(), reason.to_owned(), 0, 0, 0, 0),
        Ok(()) if c.access.evidence_state != "CURRENT" =>
            ("UNKNOWN".to_owned(), "ACCESS_EVIDENCE_NOT_CURRENT".to_owned(), 0, 0, 0, 0),
        Ok(()) if c.access.access_state == "BLOCKED" =>
            ("UNAVAILABLE".to_owned(), "ACCESS_ROUTE_BLOCKED".to_owned(), 0, 0, 0, 0),
        Ok(()) if c.access.access_state == "UNKNOWN" =>
            ("UNKNOWN".to_owned(), "ACCESS_EVIDENCE_UNKNOWN".to_owned(), 0, 0, 0, 0),
        Ok(()) => {
            let mut counts = [0_u64; 3];
            let mut tick = c.access.arrival_earliest_tick;
            loop {
                match evaluate_slot(c, tick)? {
                    SlotState::Available => counts[0] += 1,
                    SlotState::Unavailable => counts[1] += 1,
                    SlotState::Unknown => counts[2] += 1,
                }
                if tick == c.access.arrival_latest_tick { break; }
                tick = tick.checked_add(1).ok_or_else(|| "arrival tick overflow".to_owned())?;
            }
            let total = counts.iter().sum::<u64>();
            if counts[0] == total {
                ("AVAILABLE".to_owned(), "ALL_POSSIBLE_ARRIVALS_FIT_OPERATIONAL_WINDOW".to_owned(), total, counts[0], counts[1], counts[2])
            } else if counts[1] == total {
                ("UNAVAILABLE".to_owned(), "NO_POSSIBLE_ARRIVAL_FITS_OPERATIONAL_WINDOW".to_owned(), total, counts[0], counts[1], counts[2])
            } else {
                ("UNKNOWN".to_owned(), "ARRIVAL_RANGE_OR_CALENDAR_EVIDENCE_INDETERMINATE".to_owned(), total, counts[0], counts[1], counts[2])
            }
        }
    };
    Ok(OpportunityResult {
        opportunity_id: c.opportunity_id.clone(),
        scope_digest: c.scope.scope_digest.clone(),
        service_id: c.scope.service_id.clone(),
        functional_unit_id: c.scope.functional_unit_id.clone(),
        period_id: c.scope.period_id.clone(),
        time_basis_id: c.scope.time_basis_id.clone(),
        disposition,
        reason_code: reason,
        access_state: c.access.access_state.clone(),
        calendar_coverage_state: c.calendar.coverage_state.clone(),
        arrival_earliest_tick: c.access.arrival_earliest_tick,
        arrival_latest_tick: c.access.arrival_latest_tick,
        required_service_duration_ticks: c.required_service_duration_ticks,
        service_finish_latest_tick: finish,
        possible_arrival_count: possible,
        available_arrival_count: available,
        unavailable_arrival_count: unavailable,
        unknown_arrival_count: unknown,
        service_owner_revision: c.calendar.owner_revision.clone(),
        service_calendar_evidence_state: c.calendar.evidence_state.clone(),
        access_owner_revision: c.access.owner_revision.clone(),
        access_evidence_state: c.access.evidence_state.clone(),
    })
}

fn evaluate_fixture(bytes: &[u8]) -> Result<EvaluationOutput, String> {
    let fixtures = parse_fixture(bytes)?;
    validate_fixture(&fixtures)?;
    let mut results = Vec::with_capacity(fixtures.cases.len());
    let mut counts = DispositionCounts { available: 0, unavailable: 0, unknown: 0, not_comparable: 0 };
    for c in &fixtures.cases {
        let r = evaluate_case(c)?;
        let actual = ExpectedResult {
            disposition: r.disposition.clone(),
            reason_code: r.reason_code.clone(),
            possible_arrival_count: r.possible_arrival_count,
            available_arrival_count: r.available_arrival_count,
            unavailable_arrival_count: r.unavailable_arrival_count,
            unknown_arrival_count: r.unknown_arrival_count,
        };
        if actual != c.expected {
            return Err(format!("frozen oracle mismatch for {}: actual={actual:?}, expected={:?}", c.opportunity_id, c.expected));
        }
        match r.disposition.as_str() {
            "AVAILABLE" => counts.available += 1,
            "UNAVAILABLE" => counts.unavailable += 1,
            "UNKNOWN" => counts.unknown += 1,
            "NOT_COMPARABLE" => counts.not_comparable += 1,
            other => return Err(format!("unsupported disposition {other}")),
        }
        results.push(r);
    }
    let digest = Sha256::digest(bytes);
    let fixture_sha256 = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
    Ok(EvaluationOutput {
        schema: "luminous.temporal-amenity-opportunity-evaluation".to_owned(),
        schema_version: EXPECTED_VERSION.to_owned(),
        maturity: EXPECTED_MATURITY.to_owned(),
        claim_ceiling: fixtures.claim_ceiling,
        fixture_sha256,
        time_basis_tick_seconds: fixtures.time_basis_contract.tick_seconds,
        time_basis_origin: fixtures.time_basis_contract.origin.clone(),
        interval_semantics: fixtures.time_basis_contract.interval_semantics.clone(),
        opportunity_count: results.len(),
        disposition_counts: counts,
        results,
    })
}

pub(super) fn run() {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        None => FIXTURE_PATH.to_owned(),
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!("Usage: temporal-amenity-opportunity [FIXTURES.json]");
            println!("Defaults to {FIXTURE_PATH}");
            return;
        }
        Some(arg) => arg,
    };
    if args.next().is_some() {
        eprintln!("expected at most one fixture path");
        process::exit(2);
    }
    let bytes = fs::read(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        process::exit(2);
    });
    match evaluate_fixture(&bytes) {
        Ok(output) => println!("{}", serde_json::to_string_pretty(&output).expect("output serialization")),
        Err(error) => { eprintln!("temporal opportunity audit blocked: {error}"); process::exit(1); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../../fixtures/TEMPORAL_AMENITY_OPPORTUNITY_FIXTURES_V0.1.json");

    fn fixture_value() -> Value { serde_json::from_str(FIXTURE).expect("fixture JSON") }
    fn evaluate_value(v: Value) -> Result<EvaluationOutput, String> {
        evaluate_fixture(&serde_json::to_vec(&v).expect("test JSON"))
    }
    fn by_id<'a>(out: &'a EvaluationOutput, id: &str) -> &'a OpportunityResult {
        out.results.iter().find(|r| r.opportunity_id == id).expect("result exists")
    }

    #[test]
    fn frozen_interval_join_corpus_matches_every_golden_result() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixture corpus should pass");
        assert_eq!(out.opportunity_count, 20);
        assert_eq!(out.disposition_counts.available, 2);
        assert_eq!(out.disposition_counts.unavailable, 7);
        assert_eq!(out.disposition_counts.unknown, 8);
        assert_eq!(out.disposition_counts.not_comparable, 3);
        assert_eq!(out.fixture_sha256.len(), 64);
    }
    #[test]
    fn half_open_close_boundary_is_unavailable() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "ARRIVAL_EXACTLY_AT_CLOSE").disposition, "UNAVAILABLE");
    }
    #[test]
    fn service_duration_must_fit_before_close() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "SERVICE_DURATION_CROSSES_CLOSE").disposition, "UNAVAILABLE");
    }
    #[test]
    fn adjacent_current_operational_windows_cover_continuous_visit() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "CONTIGUOUS_OPERATIONAL_WINDOWS").disposition, "AVAILABLE");
    }
    #[test]
    fn complete_calendar_gap_is_known_unavailability() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "COMPLETE_CALENDAR_CLOSED_GAP").disposition, "UNAVAILABLE");
    }
    #[test]
    fn incomplete_calendar_gap_remains_unknown() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "INCOMPLETE_CALENDAR_GAP").disposition, "UNKNOWN");
    }
    #[test]
    fn stale_availability_evidence_is_not_promoted_to_available() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "STALE_OPERATIONAL_WINDOW").disposition, "UNKNOWN");
    }
    #[test]
    fn arrival_uncertainty_straddling_closure_is_unknown() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        let r = by_id(&out, "ARRIVAL_RANGE_STRADDLES_CLOSURE");
        assert_eq!(r.disposition, "UNKNOWN");
        assert_eq!(r.available_arrival_count, 1);
        assert_eq!(r.unavailable_arrival_count, 1);
    }
    #[test]
    fn scope_and_time_basis_mismatch_are_not_comparable() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "ACCESS_SCOPE_MISMATCH").disposition, "NOT_COMPARABLE");
        assert_eq!(by_id(&out, "SERVICE_CLOCK_MISMATCH").disposition, "NOT_COMPARABLE");
    }
    #[test]
    fn blocked_and_unknown_access_states_remain_distinct() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "ACCESS_ROUTE_BLOCKED").disposition, "UNAVAILABLE");
        assert_eq!(by_id(&out, "ACCESS_ROUTE_UNKNOWN").disposition, "UNKNOWN");
    }
    #[test]
    fn maintenance_and_not_commissioned_are_unavailable() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "MAINTENANCE_DURING_REQUEST").disposition, "UNAVAILABLE");
        assert_eq!(by_id(&out, "NOT_COMMISSIONED_SERVICE").disposition, "UNAVAILABLE");
    }
    #[test]
    fn complete_calendar_without_window_is_unavailable() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "COMPLETE_CALENDAR_NO_SERVICE_WINDOW").disposition, "UNAVAILABLE");
    }
    #[test]
    fn incomplete_calendar_without_window_is_unknown() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        assert_eq!(by_id(&out, "INCOMPLETE_CALENDAR_NO_WINDOW").disposition, "UNKNOWN");
    }
    #[test]
    fn mixed_owner_revisions_are_not_comparable() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        let r = by_id(&out, "MIXED_OWNER_REVISIONS");
        assert_eq!(r.disposition, "NOT_COMPARABLE");
        assert_eq!(r.reason_code, "MIXED_SERVICE_OWNER_REVISIONS");
    }
    #[test]
    fn stale_and_unknown_access_evidence_never_becomes_available() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures should pass");
        for id in [
            "STALE_QUALIFIED_ACCESS_RESULT",
            "UNKNOWN_ACCESS_EVIDENCE_QUALITY",
            "STALE_BLOCKED_ACCESS_RESULT",
        ] {
            let result = by_id(&out, id);
            assert_eq!(result.disposition, "UNKNOWN");
            assert_eq!(result.reason_code, "ACCESS_EVIDENCE_NOT_CURRENT");
        }
    }

    #[test]
    fn currentness_is_reported_separately_for_access_and_service_evidence() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures should pass");
        let result = by_id(&out, "OPEN_FULL_SERVICE_SLOT");
        assert_eq!(result.access_evidence_state, "CURRENT");
        assert_eq!(result.service_calendar_evidence_state, "CURRENT");
        let stale = by_id(&out, "STALE_QUALIFIED_ACCESS_RESULT");
        assert_eq!(stale.access_evidence_state, "STALE");
    }

    #[test]
    fn alternate_tick_duration_cannot_reuse_the_frozen_time_basis_id() {
        let mut v = fixture_value();
        v["time_basis_contract"]["tick_seconds"] = Value::from(1_800_u64);
        let e = evaluate_value(v).expect_err("altered tick scale must fail closed");
        assert!(e.contains("900-second tick profile"));
    }

    #[test]
    fn overlapping_windows_are_rejected() {
        let mut v = fixture_value();
        v["cases"][9]["calendar"]["service_windows"][1]["start_tick"] = Value::from(35_i64);
        let e = evaluate_value(v).expect_err("overlap must fail");
        assert!(e.contains("overlapping or contradictory"));
    }
    #[test]
    fn duplicate_window_identity_is_rejected() {
        let mut v = fixture_value();
        let first = v["cases"][0]["calendar"]["service_windows"][0].clone();
        v["cases"][0]["calendar"]["service_windows"].as_array_mut().expect("array").push(first);
        let e = evaluate_value(v).expect_err("duplicate identity must fail");
        assert!(e.contains("duplicate service-window identity"));
    }
    #[test]
    fn zero_and_negative_durations_are_rejected() {
        for duration in [-1_i64, 0_i64] {
            let mut v = fixture_value();
            v["cases"][0]["required_service_duration_ticks"] = Value::from(duration);
            let e = evaluate_value(v).expect_err("invalid duration must fail");
            assert!(e.contains("duration must be positive"));
        }
    }
    #[test]
    fn request_beyond_horizon_is_rejected() {
        let mut v = fixture_value();
        v["cases"][0]["access"]["arrival_latest_tick"] = Value::from(95_i64);
        let e = evaluate_value(v).expect_err("out-of-horizon request must fail");
        assert!(e.contains("exceeds scope horizon"));
    }
    #[test]
    fn excessively_wide_arrival_uncertainty_is_rejected() {
        let mut v = fixture_value();
        v["cases"][0]["scope"]["horizon_end_tick"] = Value::from(3_000_i64);
        v["cases"][0]["access"]["arrival_earliest_tick"] = Value::from(0_i64);
        v["cases"][0]["access"]["arrival_latest_tick"] = Value::from(2_000_i64);
        let e = evaluate_value(v).expect_err("wide arrival range must fail");
        assert!(e.contains("arrival uncertainty exceeds"));
    }
    #[test]
    fn window_order_is_irrelevant_for_non_overlapping_intervals() {
        let mut v = fixture_value();
        v["cases"][9]["calendar"]["service_windows"].as_array_mut().expect("array").reverse();
        let out = evaluate_value(v).expect("reordered windows should pass");
        assert_eq!(by_id(&out, "CONTIGUOUS_OPERATIONAL_WINDOWS").disposition, "AVAILABLE");
    }
    #[test]
    fn duplicate_json_object_keys_are_rejected() {
        let duplicate = br#"{"schema":"luminous.temporal-amenity-opportunity-fixtures","schema":"luminous.temporal-amenity-opportunity-fixtures"}"#;
        let e = evaluate_fixture(duplicate).expect_err("duplicate key must fail");
        assert!(e.contains("duplicate JSON object key"));
    }
    #[test]
    fn output_does_not_create_bookings_or_allocations() {
        let out = evaluate_fixture(FIXTURE.as_bytes()).expect("fixtures pass");
        let json = serde_json::to_string(&out).expect("output JSON");
        assert!(!json.contains("booking_id"));
        assert!(!json.contains("reservation_id"));
        assert!(!json.contains("assigned_capacity"));
        assert!(!json.contains("allocation_decision"));
    }
}

}

fn main() {
    implementation::run();
}
