//! Standalone source-request auditor for domain-level energy dispatch schedules.
//!
//! This crate deliberately does not depend on the LP translator, LP parser,
//! solver variable names or solver result types. It evaluates a schedule DTO
//! directly against the source request contract.

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

pub const AUDITOR_ID: &str = "energy-dispatch-schedule-auditor";
pub const AUDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SOURCE_SCHEMA_VERSION: &str = "energy-dispatch-v0.1";
pub const SCHEDULE_SCHEMA_VERSION: &str = "energy-dispatch-schedule-v0.1";
pub const MAX_REQUEST_BYTES: usize = 1_048_576;
pub const MAX_SCHEDULE_BYTES: usize = 1_048_576;
pub const MAX_INTERVALS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditError(pub String);

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AuditError {}

fn audit_error(message: impl Into<String>) -> AuditError {
    AuditError(message.into())
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn current_binary_sha256() -> Result<String, AuditError> {
    let executable = std::env::current_exe()
        .map_err(|error| audit_error(format!("resolve auditor executable: {error}")))?;
    let bytes = std::fs::read(&executable)
        .map_err(|error| audit_error(format!("read auditor executable for provenance: {error}")))?;
    Ok(sha256(&bytes))
}

struct StrictJsonSeed;

impl<'de> DeserializeSeed<'de> for StrictJsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonValueVisitor)
    }
}

struct StrictJsonValueVisitor;

impl<'de> Visitor<'de> for StrictJsonValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique member names")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("JSON numeric value must be finite"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(StrictJsonSeed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object member {key:?}"
                )));
            }
            let value = object.next_value_seed(StrictJsonSeed)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

/// Strictly parses a single JSON value and rejects duplicate object keys and
/// trailing content before typed deserialization.
pub fn parse_unique_json(input: &[u8]) -> Result<Value, AuditError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let value = StrictJsonSeed
        .deserialize(&mut deserializer)
        .map_err(|error| audit_error(format!("strict JSON parse failed: {error}")))?;
    deserializer
        .end()
        .map_err(|error| audit_error(format!("trailing JSON content: {error}")))?;
    Ok(value)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRequest {
    schema_version: String,
    case_id: String,
    synthetic: bool,
    evidence_class: String,
    evidence_ref: Option<String>,
    evidence_as_of: Option<String>,
    energy_quantum_kwh: f64,
    max_candidate_expansions: u64,
    battery: Battery,
    intervals: Vec<SourceInterval>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Battery {
    capacity_quanta: u16,
    initial_soc_quanta: u16,
    charge_input_limit_quanta: u16,
    discharge_withdrawal_limit_quanta: u16,
    charge_efficiency_numerator: u16,
    charge_efficiency_denominator: u16,
    discharge_efficiency_numerator: u16,
    discharge_efficiency_denominator: u16,
    terminal_reserve_quanta: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInterval {
    interval_id: String,
    start_unix_seconds: i64,
    duration_seconds: u64,
    grid_outage: bool,
    pv_available_quanta: u16,
    grid_import_limit_quanta: u16,
    essential_demand_quanta: u16,
    flexible_demand_quanta: u16,
    grid_cost_units_per_quantum: u32,
    grid_carbon_grams_per_quantum: u32,
}

/// Domain-level dispatch trajectory. No LP variable names or solver
/// intermediates appear in this public schedule contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DispatchSchedule {
    pub schema_version: String,
    pub case_id: String,
    pub synthetic_source: bool,
    pub request_sha256: String,
    pub interval_dispatches: Vec<IntervalDispatch>,
    pub terminal_soc_quanta: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IntervalDispatch {
    pub interval_id: String,
    pub grid_import_quanta: u16,
    pub charge_input_quanta: u16,
    pub discharge_withdrawal_quanta: u16,
    pub soc_after_quanta: u16,
    pub essential_served_quanta: u16,
    pub essential_unserved_quanta: u16,
    pub flexible_served_quanta: u16,
    pub flexible_unserved_quanta: u16,
    pub pv_used_quanta: u16,
    pub pv_curtailed_quanta: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditDisposition {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditFinding {
    pub predicate: String,
    pub interval_index: Option<usize>,
    pub interval_id: Option<String>,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecomputedObjectives {
    pub essential_unserved_quanta: Option<i64>,
    pub flexible_unserved_quanta: Option<i64>,
    pub operating_cost_units: Option<i64>,
    pub grid_carbon_grams: Option<i64>,
    pub curtailed_pv_quanta: Option<i64>,
    pub terminal_soc_quanta: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReceipt {
    pub receipt_schema: String,
    pub auditor_id: String,
    pub auditor_version: String,
    pub auditor_binary_sha256: String,
    pub source_schema_version: String,
    pub schedule_schema_version: String,
    pub case_id: String,
    pub synthetic_source: bool,
    pub max_candidate_expansions: u64,
    pub request_sha256: String,
    pub schedule_sha256: String,
    pub interval_count: usize,
    pub disposition: AuditDisposition,
    pub findings: Vec<AuditFinding>,
    pub recomputed_objectives: RecomputedObjectives,
    pub receipt_sha256: String,
    pub claim_ceiling: String,
}

#[derive(Serialize)]
struct ReceiptSubject<'a> {
    receipt_schema: &'a str,
    auditor_id: &'a str,
    auditor_version: &'a str,
    auditor_binary_sha256: &'a str,
    source_schema_version: &'a str,
    schedule_schema_version: &'a str,
    case_id: &'a str,
    synthetic_source: bool,
    max_candidate_expansions: u64,
    request_sha256: &'a str,
    schedule_sha256: &'a str,
    interval_count: usize,
    disposition: AuditDisposition,
    findings: &'a [AuditFinding],
    recomputed_objectives: &'a RecomputedObjectives,
    claim_ceiling: &'a str,
}

fn add_finding(
    findings: &mut Vec<AuditFinding>,
    predicate: &str,
    interval_index: Option<usize>,
    interval_id: Option<&str>,
    passed: bool,
    detail: impl Into<String>,
) {
    findings.push(AuditFinding {
        predicate: predicate.to_owned(),
        interval_index,
        interval_id: interval_id.map(str::to_owned),
        passed,
        detail: detail.into(),
    });
}

fn validate_request(request: &SourceRequest, findings: &mut Vec<AuditFinding>) {
    add_finding(
        findings,
        "source-schema-version",
        None,
        None,
        request.schema_version == SOURCE_SCHEMA_VERSION,
        format!(
            "source schema version is {}; expected {}",
            request.schema_version, SOURCE_SCHEMA_VERSION
        ),
    );
    add_finding(
        findings,
        "case-id",
        None,
        None,
        !request.case_id.trim().is_empty(),
        "source case ID must be non-empty",
    );
    add_finding(
        findings,
        "evidence-class",
        None,
        None,
        !request.evidence_class.trim().is_empty(),
        "evidence class must be non-empty",
    );
    add_finding(
        findings,
        "evidence-reference-shape",
        None,
        None,
        request.evidence_ref.as_ref().is_none_or(|value| !value.trim().is_empty()),
        "optional evidence reference, when supplied, must be non-empty",
    );
    add_finding(
        findings,
        "evidence-as-of",
        None,
        None,
        request.evidence_as_of.as_ref().is_none_or(|value| !value.trim().is_empty()),
        "optional evidence timestamp, when supplied, must be non-empty",
    );
    add_finding(
        findings,
        "energy-quantum",
        None,
        None,
        request.energy_quantum_kwh.is_finite() && request.energy_quantum_kwh > 0.0,
        "energy_quantum_kwh must be finite and positive",
    );
    add_finding(
        findings,
        "candidate-expansion-limit",
        None,
        None,
        (1..=1_000_000).contains(&request.max_candidate_expansions),
        "max_candidate_expansions must be within [1, 1000000]",
    );
    add_finding(
        findings,
        "evidence-reference",
        None,
        None,
        request.evidence_class == "analyst_assumption"
            || request.evidence_ref.as_deref().is_some_and(|value| !value.trim().is_empty()),
        "non-assumption evidence requires a non-empty evidence_ref",
    );
    add_finding(
        findings,
        "interval-count",
        None,
        None,
        !request.intervals.is_empty() && request.intervals.len() <= MAX_INTERVALS,
        format!("interval count must be between 1 and {MAX_INTERVALS}"),
    );

    let battery = &request.battery;
    add_finding(
        findings,
        "battery-initial-soc",
        None,
        None,
        battery.initial_soc_quanta <= battery.capacity_quanta,
        "initial SOC must not exceed battery capacity",
    );
    add_finding(
        findings,
        "battery-terminal-reserve",
        None,
        None,
        battery.terminal_reserve_quanta <= battery.capacity_quanta,
        "terminal reserve must not exceed battery capacity",
    );
    add_finding(
        findings,
        "battery-charge-efficiency",
        None,
        None,
        battery.charge_efficiency_denominator > 0
            && battery.charge_efficiency_numerator > 0
            && battery.charge_efficiency_numerator <= battery.charge_efficiency_denominator,
        "charge efficiency must be a positive rational no greater than one",
    );
    add_finding(
        findings,
        "battery-discharge-efficiency",
        None,
        None,
        battery.discharge_efficiency_denominator > 0
            && battery.discharge_efficiency_numerator > 0
            && battery.discharge_efficiency_numerator <= battery.discharge_efficiency_denominator,
        "discharge efficiency must be a positive rational no greater than one",
    );

    let mut seen_ids = BTreeSet::new();
    let mut expected_start: Option<i64> = None;
    for (index, interval) in request.intervals.iter().enumerate() {
        let id_nonempty = !interval.interval_id.trim().is_empty();
        add_finding(
            findings,
            "interval-id",
            Some(index),
            Some(&interval.interval_id),
            id_nonempty && seen_ids.insert(interval.interval_id.clone()),
            "interval IDs must be non-empty and unique",
        );
        let duration = i64::try_from(interval.duration_seconds);
        add_finding(
            findings,
            "interval-duration",
            Some(index),
            Some(&interval.interval_id),
            interval.duration_seconds > 0 && duration.is_ok(),
            "interval duration must be positive and fit the signed chronology range",
        );
        let contiguous = expected_start.is_none_or(|expected| {
            expected == interval.start_unix_seconds
        });
        add_finding(
            findings,
            "interval-chronology",
            Some(index),
            Some(&interval.interval_id),
            contiguous,
            "intervals must be contiguous with no gaps, overlaps or reordering",
        );
        expected_start = duration.ok().and_then(|duration| {
            interval.start_unix_seconds.checked_add(duration)
        });
        if duration.is_ok() && expected_start.is_none() {
            add_finding(
                findings,
                "interval-chronology-overflow",
                Some(index),
                Some(&interval.interval_id),
                false,
                "interval end timestamp must fit signed 64-bit chronology",
            );
        }
    }
}

fn checked_floor_product(
    amount: u16,
    numerator: u16,
    denominator: u16,
) -> Option<i64> {
    if denominator == 0 {
        return None;
    }
    i64::from(amount)
        .checked_mul(i64::from(numerator))
        .map(|product| product / i64::from(denominator))
}

fn recompute_objectives(
    request: &SourceRequest,
    schedule: &DispatchSchedule,
) -> Result<RecomputedObjectives, AuditError> {
    if schedule.interval_dispatches.len() != request.intervals.len() {
        // A partial trajectory cannot support complete objective values.
        // Return explicit nulls and let the interval-count finding produce a
        // rejected receipt rather than mislabeling this as an auditor failure.
        return Ok(RecomputedObjectives {
            essential_unserved_quanta: None,
            flexible_unserved_quanta: None,
            operating_cost_units: None,
            grid_carbon_grams: None,
            curtailed_pv_quanta: None,
            terminal_soc_quanta: None,
        });
    }
    let mut essential_unserved = 0_i64;
    let mut flexible_unserved = 0_i64;
    let mut operating_cost = 0_i64;
    let mut grid_carbon = 0_i64;
    let mut curtailed_pv = 0_i64;

    let mut recomputed_terminal_soc = Some(i64::from(request.battery.initial_soc_quanta));
    for (interval, dispatch) in request.intervals.iter().zip(&schedule.interval_dispatches) {
        essential_unserved = essential_unserved
            .checked_add(i64::from(dispatch.essential_unserved_quanta))
            .ok_or_else(|| audit_error("essential unserved objective overflow"))?;
        flexible_unserved = flexible_unserved
            .checked_add(i64::from(dispatch.flexible_unserved_quanta))
            .ok_or_else(|| audit_error("flexible unserved objective overflow"))?;
        operating_cost = operating_cost
            .checked_add(
                i64::from(dispatch.grid_import_quanta)
                    .checked_mul(i64::from(interval.grid_cost_units_per_quantum))
                    .ok_or_else(|| audit_error("operating cost product overflow"))?,
            )
            .ok_or_else(|| audit_error("operating cost objective overflow"))?;
        grid_carbon = grid_carbon
            .checked_add(
                i64::from(dispatch.grid_import_quanta)
                    .checked_mul(i64::from(interval.grid_carbon_grams_per_quantum))
                    .ok_or_else(|| audit_error("grid carbon product overflow"))?,
            )
            .ok_or_else(|| audit_error("grid carbon objective overflow"))?;
        curtailed_pv = curtailed_pv
            .checked_add(i64::from(dispatch.pv_curtailed_quanta))
            .ok_or_else(|| audit_error("curtailed PV objective overflow"))?;

        if let Some(current_soc) = recomputed_terminal_soc {
            let gain = checked_floor_product(
                dispatch.charge_input_quanta,
                request.battery.charge_efficiency_numerator,
                request.battery.charge_efficiency_denominator,
            );
            let withdrawal = i64::from(dispatch.discharge_withdrawal_quanta);
            recomputed_terminal_soc = gain.and_then(|gain| {
                current_soc
                    .checked_add(gain)
                    .and_then(|value| value.checked_sub(withdrawal))
            });
        }
    }

    let terminal_soc = recomputed_terminal_soc;
    Ok(RecomputedObjectives {
        essential_unserved_quanta: Some(essential_unserved),
        flexible_unserved_quanta: Some(flexible_unserved),
        operating_cost_units: Some(operating_cost),
        grid_carbon_grams: Some(grid_carbon),
        curtailed_pv_quanta: Some(curtailed_pv),
        terminal_soc_quanta: terminal_soc,
    })
}

fn validate_schedule(
    request: &SourceRequest,
    schedule: &DispatchSchedule,
    request_sha256: &str,
    findings: &mut Vec<AuditFinding>,
) {
    add_finding(
        findings,
        "schedule-schema-version",
        None,
        None,
        schedule.schema_version == SCHEDULE_SCHEMA_VERSION,
        format!(
            "schedule schema version is {}; expected {}",
            schedule.schema_version, SCHEDULE_SCHEMA_VERSION
        ),
    );
    add_finding(
        findings,
        "schedule-case-id",
        None,
        None,
        schedule.case_id == request.case_id,
        "schedule case ID must equal the source request case ID",
    );
    add_finding(
        findings,
        "schedule-request-digest",
        None,
        None,
        schedule.request_sha256 == request_sha256,
        "schedule request SHA-256 must bind the exact source request bytes",
    );
    add_finding(
        findings,
        "schedule-interval-count",
        None,
        None,
        schedule.interval_dispatches.len() == request.intervals.len(),
        "schedule must contain exactly one row for each source interval",
    );

    let battery = &request.battery;
    let mut previous_soc = i64::from(battery.initial_soc_quanta);
    let mut terminal_soc = previous_soc;

    for (index, interval) in request.intervals.iter().enumerate() {
        let Some(dispatch) = schedule.interval_dispatches.get(index) else {
            add_finding(
                findings,
                "schedule-interval-present",
                Some(index),
                Some(&interval.interval_id),
                false,
                "schedule is missing a dispatch row for this source interval",
            );
            continue;
        };

        add_finding(
            findings,
            "schedule-interval-id",
            Some(index),
            Some(&interval.interval_id),
            dispatch.interval_id == interval.interval_id,
            "schedule interval ID and order must match the source interval",
        );

        let grid_limit = if interval.grid_outage {
            0_i64
        } else {
            i64::from(interval.grid_import_limit_quanta)
        };
        let grid_import = i64::from(dispatch.grid_import_quanta);
        add_finding(
            findings,
            "grid-import-limit",
            Some(index),
            Some(&interval.interval_id),
            grid_import <= grid_limit,
            if interval.grid_outage {
                "grid import must be zero during an outage"
            } else {
                "grid import must not exceed the source interval limit"
            },
        );

        let charge = i64::from(dispatch.charge_input_quanta);
        let withdrawal = i64::from(dispatch.discharge_withdrawal_quanta);
        add_finding(
            findings,
            "battery-charge-limit",
            Some(index),
            Some(&interval.interval_id),
            charge <= i64::from(battery.charge_input_limit_quanta),
            "charge input must not exceed the battery charge limit",
        );
        add_finding(
            findings,
            "battery-discharge-limit",
            Some(index),
            Some(&interval.interval_id),
            withdrawal <= i64::from(
                battery
                    .discharge_withdrawal_limit_quanta
                    .min(battery.capacity_quanta),
            ),
            "discharge withdrawal must not exceed the battery discharge limit",
        );
        add_finding(
            findings,
            "battery-mode-exclusivity",
            Some(index),
            Some(&interval.interval_id),
            !(charge > 0 && withdrawal > 0),
            "charging and discharging may not occur in the same interval",
        );
        add_finding(
            findings,
            "battery-available-soc",
            Some(index),
            Some(&interval.interval_id),
            withdrawal <= previous_soc,
            "discharge withdrawal must not exceed SOC at the interval start",
        );

        let charge_gain = checked_floor_product(
            dispatch.charge_input_quanta,
            battery.charge_efficiency_numerator,
            battery.charge_efficiency_denominator,
        );
        let discharge_delivered = checked_floor_product(
            dispatch.discharge_withdrawal_quanta,
            battery.discharge_efficiency_numerator,
            battery.discharge_efficiency_denominator,
        );
        add_finding(
            findings,
            "charge-efficiency-floor",
            Some(index),
            Some(&interval.interval_id),
            charge_gain.is_some(),
            "charge gain is recomputed using exact integer floor efficiency",
        );
        add_finding(
            findings,
            "discharge-efficiency-floor",
            Some(index),
            Some(&interval.interval_id),
            discharge_delivered.is_some(),
            "discharge delivery is recomputed using exact integer floor efficiency",
        );

        let expected_soc = charge_gain
            .zip(discharge_delivered)
            .and_then(|(gain, _)| previous_soc.checked_add(gain))
            .and_then(|stored| stored.checked_sub(withdrawal));
        let actual_soc = i64::from(dispatch.soc_after_quanta);
        add_finding(
            findings,
            "soc-transition",
            Some(index),
            Some(&interval.interval_id),
            expected_soc == Some(actual_soc),
            "SOC after the interval must equal start SOC + floor(charge gain) - withdrawal",
        );
        add_finding(
            findings,
            "soc-capacity",
            Some(index),
            Some(&interval.interval_id),
            actual_soc <= i64::from(battery.capacity_quanta),
            "SOC after the interval must not exceed battery capacity",
        );

        let essential_total = i64::from(dispatch.essential_served_quanta)
            .checked_add(i64::from(dispatch.essential_unserved_quanta));
        add_finding(
            findings,
            "essential-load-partition",
            Some(index),
            Some(&interval.interval_id),
            essential_total == Some(i64::from(interval.essential_demand_quanta)),
            "essential served plus unserved must equal source essential demand",
        );
        let flexible_total = i64::from(dispatch.flexible_served_quanta)
            .checked_add(i64::from(dispatch.flexible_unserved_quanta));
        add_finding(
            findings,
            "flexible-load-partition",
            Some(index),
            Some(&interval.interval_id),
            flexible_total == Some(i64::from(interval.flexible_demand_quanta)),
            "flexible served plus unserved must equal source flexible demand",
        );
        let pv_total = i64::from(dispatch.pv_used_quanta)
            .checked_add(i64::from(dispatch.pv_curtailed_quanta));
        add_finding(
            findings,
            "pv-partition",
            Some(index),
            Some(&interval.interval_id),
            pv_total == Some(i64::from(interval.pv_available_quanta)),
            "PV used plus curtailed must equal source PV availability",
        );

        let supply = i64::from(dispatch.pv_used_quanta)
            .checked_add(grid_import)
            .and_then(|value| value.checked_add(discharge_delivered.unwrap_or(-1)));
        let demand = i64::from(dispatch.essential_served_quanta)
            .checked_add(i64::from(dispatch.flexible_served_quanta))
            .and_then(|value| value.checked_add(charge));
        add_finding(
            findings,
            "bus-energy-balance",
            Some(index),
            Some(&interval.interval_id),
            supply.is_some() && discharge_delivered.is_some() && supply == demand,
            "PV used + grid import + floor(discharge delivery) must equal served loads + charge input",
        );

        if let Some(value) = expected_soc {
            terminal_soc = value;
            previous_soc = value;
        } else {
            // Keep the independently propagated state when the transition
            // arithmetic is invalid; never trust a candidate SOC to repair it.
            terminal_soc = previous_soc;
        }
    }

    add_finding(
        findings,
        "schedule-terminal-soc",
        None,
        None,
        i64::from(schedule.terminal_soc_quanta) == terminal_soc,
        "schedule terminal SOC must equal the final independently recomputed state",
    );
    add_finding(
        findings,
        "terminal-reserve",
        None,
        None,
        terminal_soc >= i64::from(battery.terminal_reserve_quanta),
        "terminal SOC must meet or exceed the source terminal reserve",
    );
}

fn build_receipt(
    request: &SourceRequest,
    schedule: &DispatchSchedule,
    request_sha256: String,
    schedule_sha256: String,
    findings: Vec<AuditFinding>,
    objectives: RecomputedObjectives,
) -> Result<AuditReceipt, AuditError> {
    let disposition = if findings.iter().all(|finding| finding.passed) {
        AuditDisposition::Accepted
    } else {
        AuditDisposition::Rejected
    };
    let receipt_schema = "energy-dispatch-schedule-audit-receipt-v0.1";
    let auditor_binary_sha256 = current_binary_sha256()?;
    let claim_ceiling = concat!(
        "independent source-request schedule conformance for the declared integer ",
        "battery/PV/grid dispatch model only; no claim of physical-model completeness, ",
        "input evidence authenticity, optimality, real-site reliability, field safety ",
        "or execution authorization"
    );

    let subject = ReceiptSubject {
        receipt_schema,
        auditor_id: AUDITOR_ID,
        auditor_version: AUDITOR_VERSION,
        auditor_binary_sha256: &auditor_binary_sha256,
        source_schema_version: &request.schema_version,
        schedule_schema_version: &schedule.schema_version,
        case_id: &request.case_id,
        synthetic_source: request.synthetic,
        max_candidate_expansions: request.max_candidate_expansions,
        request_sha256: &request_sha256,
        schedule_sha256: &schedule_sha256,
        interval_count: schedule.interval_dispatches.len(),
        disposition,
        findings: &findings,
        recomputed_objectives: &objectives,
        claim_ceiling,
    };
    let subject_bytes = serde_json::to_vec(&subject)
        .map_err(|error| audit_error(format!("serialize receipt subject: {error}")))?;

    Ok(AuditReceipt {
        receipt_schema: receipt_schema.to_owned(),
        auditor_id: AUDITOR_ID.to_owned(),
        auditor_version: AUDITOR_VERSION.to_owned(),
        auditor_binary_sha256,
        source_schema_version: request.schema_version.clone(),
        schedule_schema_version: schedule.schema_version.clone(),
        case_id: request.case_id.clone(),
        synthetic_source: request.synthetic,
        max_candidate_expansions: request.max_candidate_expansions,
        request_sha256,
        schedule_sha256,
        interval_count: schedule.interval_dispatches.len(),
        disposition,
        findings,
        recomputed_objectives: objectives,
        receipt_sha256: sha256(&subject_bytes),
        claim_ceiling: claim_ceiling.to_owned(),
    })
}

/// Audit exact source request and schedule bytes. The request digest in the
/// schedule must equal SHA-256 of the original request bytes.
pub fn audit_schedule_bytes(
    request_bytes: &[u8],
    schedule_bytes: &[u8],
) -> Result<AuditReceipt, AuditError> {
    if request_bytes.len() > MAX_REQUEST_BYTES {
        return Err(audit_error(format!(
            "request exceeds {MAX_REQUEST_BYTES} byte limit"
        )));
    }
    if schedule_bytes.len() > MAX_SCHEDULE_BYTES {
        return Err(audit_error(format!(
            "schedule exceeds {MAX_SCHEDULE_BYTES} byte limit"
        )));
    }
    let request_value = parse_unique_json(request_bytes)?;
    let schedule_value = parse_unique_json(schedule_bytes)?;
    let request: SourceRequest = serde_json::from_value(request_value)
        .map_err(|error| audit_error(format!("request fields do not match schema: {error}")))?;
    let schedule: DispatchSchedule = serde_json::from_value(schedule_value)
        .map_err(|error| audit_error(format!("schedule fields do not match schema: {error}")))?;

    let request_sha256 = sha256(request_bytes);
    let schedule_sha256 = sha256(schedule_bytes);
    let mut findings = Vec::new();
    validate_request(&request, &mut findings);
    validate_schedule(
        &request,
        &schedule,
        &request_sha256,
        &mut findings,
    );
    let objectives = recompute_objectives(&request, &schedule)?;
    build_receipt(
        &request,
        &schedule,
        request_sha256,
        schedule_sha256,
        findings,
        objectives,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_intervals(
        capacity: u16,
        initial_soc: u16,
        charge_limit: u16,
        discharge_limit: u16,
        charge_efficiency: (u16, u16),
        discharge_efficiency: (u16, u16),
        terminal_reserve: u16,
        intervals: Value,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-v0.1",
            "case_id": "schedule-audit-fixture",
            "synthetic": true,
            "evidence_class": "analyst_assumption",
            "evidence_ref": null,
            "evidence_as_of": null,
            "energy_quantum_kwh": 1.0,
            "max_candidate_expansions": 100000,
            "battery": {
                "capacity_quanta": capacity,
                "initial_soc_quanta": initial_soc,
                "charge_input_limit_quanta": charge_limit,
                "discharge_withdrawal_limit_quanta": discharge_limit,
                "charge_efficiency_numerator": charge_efficiency.0,
                "charge_efficiency_denominator": charge_efficiency.1,
                "discharge_efficiency_numerator": discharge_efficiency.0,
                "discharge_efficiency_denominator": discharge_efficiency.1,
                "terminal_reserve_quanta": terminal_reserve
            },
            "intervals": intervals
        }))
        .unwrap()
    }

    fn interval(
        id: &str,
        start: i64,
        outage: bool,
        pv: u16,
        grid_limit: u16,
        essential: u16,
        flexible: u16,
        cost: u32,
        carbon: u32,
    ) -> Value {
        serde_json::json!({
            "interval_id": id,
            "start_unix_seconds": start,
            "duration_seconds": 3600,
            "grid_outage": outage,
            "pv_available_quanta": pv,
            "grid_import_limit_quanta": grid_limit,
            "essential_demand_quanta": essential,
            "flexible_demand_quanta": flexible,
            "grid_cost_units_per_quantum": cost,
            "grid_carbon_grams_per_quantum": carbon
        })
    }

    fn step(
        id: &str,
        grid: u16,
        charge: u16,
        withdrawal: u16,
        soc_after: u16,
        essential_served: u16,
        essential_unserved: u16,
        flexible_served: u16,
        flexible_unserved: u16,
        pv_used: u16,
        pv_curtailed: u16,
    ) -> Value {
        serde_json::json!({
            "interval_id": id,
            "grid_import_quanta": grid,
            "charge_input_quanta": charge,
            "discharge_withdrawal_quanta": withdrawal,
            "soc_after_quanta": soc_after,
            "essential_served_quanta": essential_served,
            "essential_unserved_quanta": essential_unserved,
            "flexible_served_quanta": flexible_served,
            "flexible_unserved_quanta": flexible_unserved,
            "pv_used_quanta": pv_used,
            "pv_curtailed_quanta": pv_curtailed
        })
    }

    fn schedule_bytes(request_bytes: &[u8], steps: Value, terminal_soc: u16) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-schedule-v0.1",
            "case_id": "schedule-audit-fixture",
            "request_sha256": sha256(request_bytes),
            "interval_dispatches": steps,
            "terminal_soc_quanta": terminal_soc
        }))
        .unwrap()
    }

    fn h2_request() -> Vec<u8> {
        request_with_intervals(
            1,
            0,
            2,
            2,
            (1, 1),
            (1, 1),
            0,
            serde_json::json!([
                interval("charge-from-pv", 1_800_000_000, false, 1, 0, 0, 0, 3, 7),
                interval("serve-during-outage", 1_800_003_600, true, 0, 0, 1, 0, 3, 7)
            ]),
        )
    }

    fn h2_schedule(request: &[u8]) -> Vec<u8> {
        schedule_bytes(
            request,
            serde_json::json!([
                step("charge-from-pv", 0, 1, 0, 1, 0, 0, 0, 0, 1, 0),
                step("serve-during-outage", 0, 0, 1, 0, 1, 0, 0, 0, 0, 0)
            ]),
            0,
        )
    }

    #[test]
    fn accepts_known_h2_charge_then_outage_schedule_and_recomputes_objectives() {
        let request = h2_request();
        let schedule = h2_schedule(&request);
        let receipt = audit_schedule_bytes(&request, &schedule).unwrap();
        assert_eq!(receipt.disposition, AuditDisposition::Accepted);
        assert_eq!(receipt.interval_count, 2);
        assert_eq!(receipt.recomputed_objectives.essential_unserved_quanta, Some(0));
        assert_eq!(receipt.recomputed_objectives.flexible_unserved_quanta, Some(0));
        assert_eq!(receipt.recomputed_objectives.operating_cost_units, Some(0));
        assert_eq!(receipt.recomputed_objectives.grid_carbon_grams, Some(0));
        assert_eq!(receipt.recomputed_objectives.curtailed_pv_quanta, Some(0));
        assert_eq!(receipt.recomputed_objectives.terminal_soc_quanta, Some(0));
        assert_eq!(receipt.request_sha256, sha256(&request));
        assert_eq!(receipt.schedule_sha256, sha256(&schedule));
        assert_eq!(receipt.receipt_sha256.len(), 64);
        assert_eq!(receipt.auditor_binary_sha256.len(), 64);
    }

    #[test]
    fn rejects_soc_mutation_at_each_h2_interval() {
        let request = h2_request();
        let baseline: Value = serde_json::from_slice(&h2_schedule(&request)).unwrap();
        for index in 0..2 {
            let mut mutant = baseline.clone();
            mutant["interval_dispatches"][index]["soc_after_quanta"] =
                serde_json::json!(if index == 0 { 0 } else { 1 });
            let receipt = audit_schedule_bytes(
                &request,
                &serde_json::to_vec(&mutant).unwrap(),
            )
            .unwrap();
            assert_eq!(receipt.disposition, AuditDisposition::Rejected);
            assert!(receipt.findings.iter().any(|finding| {
                finding.predicate == "soc-transition" && !finding.passed
            }));
            assert_eq!(
                receipt.recomputed_objectives.terminal_soc_quanta,
                Some(0),
                "terminal SOC objective must be recomputed from physical actions, not reported SOC fields"
            );
        }
    }

    #[test]
    fn rejects_outage_import_mode_conflict_and_bad_energy_partitions() {
        let request = h2_request();
        let baseline: Value = serde_json::from_slice(&h2_schedule(&request)).unwrap();
        let mutations: Vec<(usize, &str, Value)> = vec![
            (1, "grid_import_quanta", serde_json::json!(1)),
            (0, "discharge_withdrawal_quanta", serde_json::json!(1)),
            (1, "essential_served_quanta", serde_json::json!(0)),
            (0, "pv_curtailed_quanta", serde_json::json!(1)),
        ];
        for (index, field, value) in mutations {
            let mut mutant = baseline.clone();
            mutant["interval_dispatches"][index][field] = value;
            let receipt = audit_schedule_bytes(
                &request,
                &serde_json::to_vec(&mutant).unwrap(),
            )
            .unwrap();
            assert_eq!(receipt.disposition, AuditDisposition::Rejected);
        }
    }

    #[test]
    fn rejects_schedule_reordering_and_request_cross_association() {
        let request = h2_request();
        let baseline: Value = serde_json::from_slice(&h2_schedule(&request)).unwrap();
        let mut reordered = baseline.clone();
        reordered["interval_dispatches"].as_array_mut().unwrap().swap(0, 1);
        let reordered_receipt = audit_schedule_bytes(
            &request,
            &serde_json::to_vec(&reordered).unwrap(),
        )
        .unwrap();
        assert_eq!(reordered_receipt.disposition, AuditDisposition::Rejected);
        assert!(reordered_receipt.findings.iter().any(|finding| {
            finding.predicate == "schedule-interval-id" && !finding.passed
        }));

        let mut crossed = baseline;
        crossed["request_sha256"] = serde_json::json!("0".repeat(64));
        let crossed_receipt = audit_schedule_bytes(
            &request,
            &serde_json::to_vec(&crossed).unwrap(),
        )
        .unwrap();
        assert_eq!(crossed_receipt.disposition, AuditDisposition::Rejected);
        assert!(crossed_receipt.findings.iter().any(|finding| {
            finding.predicate == "schedule-request-digest" && !finding.passed
        }));
    }

    #[test]
    fn accepts_h3_and_h4_schedules_with_idle_and_recovery_intervals() {
        let h2 = h2_request();
        let mut h3_value: Value = serde_json::from_slice(&h2).unwrap();
        h3_value["intervals"].as_array_mut().unwrap().push(interval(
            "post-outage-idle",
            1_800_007_200,
            false,
            0,
            0,
            0,
            0,
            3,
            7,
        ));
        let h3 = serde_json::to_vec(&h3_value).unwrap();
        let h3_schedule = schedule_bytes(
            &h3,
            serde_json::json!([
                step("charge-from-pv", 0, 1, 0, 1, 0, 0, 0, 0, 1, 0),
                step("serve-during-outage", 0, 0, 1, 0, 1, 0, 0, 0, 0, 0),
                step("post-outage-idle", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
            ]),
            0,
        );
        assert_eq!(
            audit_schedule_bytes(&h3, &h3_schedule).unwrap().disposition,
            AuditDisposition::Accepted
        );

        let mut h4_value: Value = serde_json::from_slice(&h3).unwrap();
        h4_value["intervals"].as_array_mut().unwrap().push(interval(
            "post-outage-recovery",
            1_800_010_800,
            false,
            1,
            1,
            0,
            0,
            3,
            7,
        ));
        let h4 = serde_json::to_vec(&h4_value).unwrap();
        let h4_schedule = schedule_bytes(
            &h4,
            serde_json::json!([
                step("charge-from-pv", 0, 1, 0, 1, 0, 0, 0, 0, 1, 0),
                step("serve-during-outage", 0, 0, 1, 0, 1, 0, 0, 0, 0, 0),
                step("post-outage-idle", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
                step("post-outage-recovery", 0, 0, 0, 0, 0, 0, 0, 0, 0, 1)
            ]),
            0,
        );
        assert_eq!(
            audit_schedule_bytes(&h4, &h4_schedule).unwrap().disposition,
            AuditDisposition::Accepted
        );
    }

    #[test]
    fn checks_non_integral_efficiency_with_exact_floor_semantics() {
        let charge_request = request_with_intervals(
            2,
            0,
            2,
            2,
            (2, 3),
            (2, 3),
            1,
            serde_json::json!([
                interval("fractional-charge", 1_800_000_000, false, 2, 0, 0, 0, 3, 7)
            ]),
        );
        let charge_schedule = schedule_bytes(
            &charge_request,
            serde_json::json!([
                step("fractional-charge", 0, 2, 0, 1, 0, 0, 0, 0, 2, 0)
            ]),
            1,
        );
        assert_eq!(
            audit_schedule_bytes(&charge_request, &charge_schedule)
                .unwrap()
                .disposition,
            AuditDisposition::Accepted
        );

        let discharge_request = request_with_intervals(
            2,
            2,
            2,
            2,
            (2, 3),
            (2, 3),
            0,
            serde_json::json!([
                interval("fractional-discharge", 1_800_000_000, false, 0, 0, 1, 0, 3, 7)
            ]),
        );
        let discharge_schedule = schedule_bytes(
            &discharge_request,
            serde_json::json!([
                step("fractional-discharge", 0, 0, 2, 0, 1, 0, 0, 0, 0, 0)
            ]),
            0,
        );
        assert_eq!(
            audit_schedule_bytes(&discharge_request, &discharge_schedule)
                .unwrap()
                .disposition,
            AuditDisposition::Accepted
        );
    }

    #[test]
    fn recomputes_nonzero_grid_cost_and_carbon_from_source_coefficients() {
        let request = request_with_intervals(
            1,
            0,
            0,
            0,
            (1, 1),
            (1, 1),
            0,
            serde_json::json!([
                interval("grid-service", 1_800_000_000, false, 0, 1, 1, 0, 3, 7)
            ]),
        );
        let schedule = schedule_bytes(
            &request,
            serde_json::json!([
                step("grid-service", 1, 0, 0, 0, 1, 0, 0, 0, 0, 0)
            ]),
            0,
        );
        let receipt = audit_schedule_bytes(&request, &schedule).unwrap();
        assert_eq!(receipt.disposition, AuditDisposition::Accepted);
        assert_eq!(receipt.recomputed_objectives.operating_cost_units, Some(3));
        assert_eq!(receipt.recomputed_objectives.grid_carbon_grams, Some(7));
    }

    #[test]
    fn emits_rejected_receipt_when_source_efficiency_prevents_terminal_objective_recomputation() {
        let mut request_value: Value = serde_json::from_slice(&h2_request()).unwrap();
        request_value["battery"]["charge_efficiency_denominator"] = serde_json::json!(0);
        let request = serde_json::to_vec(&request_value).unwrap();
        let schedule = h2_schedule(&request);
        let receipt = audit_schedule_bytes(&request, &schedule).unwrap();
        assert_eq!(receipt.disposition, AuditDisposition::Rejected);
        assert_eq!(receipt.recomputed_objectives.essential_unserved_quanta, Some(0));
        assert_eq!(receipt.recomputed_objectives.operating_cost_units, Some(0));
        assert!(receipt.recomputed_objectives.terminal_soc_quanta.is_none());
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "battery-charge-efficiency" && !finding.passed
        }));
    }

    #[test]
    fn enforces_contiguous_source_chronology_evidence_reference_and_expansion_limits() {
        let base = h2_request();
        let mut gap: Value = serde_json::from_slice(&base).unwrap();
        gap["intervals"][1]["start_unix_seconds"] = serde_json::json!(1_800_007_200_i64);
        let gap_request = serde_json::to_vec(&gap).unwrap();
        let gap_schedule = h2_schedule(&gap_request);
        let gap_receipt = audit_schedule_bytes(&gap_request, &gap_schedule).unwrap();
        assert_eq!(gap_receipt.disposition, AuditDisposition::Rejected);
        assert!(gap_receipt.findings.iter().any(|finding| {
            finding.predicate == "interval-chronology" && !finding.passed
        }));

        let mut missing_evidence: Value = serde_json::from_slice(&base).unwrap();
        missing_evidence["evidence_class"] = serde_json::json!("measured");
        missing_evidence["evidence_ref"] = Value::Null;
        let evidence_request = serde_json::to_vec(&missing_evidence).unwrap();
        let evidence_schedule = h2_schedule(&evidence_request);
        let evidence_receipt =
            audit_schedule_bytes(&evidence_request, &evidence_schedule).unwrap();
        assert_eq!(evidence_receipt.disposition, AuditDisposition::Rejected);
        assert!(evidence_receipt.findings.iter().any(|finding| {
            finding.predicate == "evidence-reference" && !finding.passed
        }));

        let mut invalid_expansion: Value = serde_json::from_slice(&base).unwrap();
        invalid_expansion["max_candidate_expansions"] = serde_json::json!(1_000_001);
        let expansion_request = serde_json::to_vec(&invalid_expansion).unwrap();
        let expansion_schedule = h2_schedule(&expansion_request);
        let expansion_receipt =
            audit_schedule_bytes(&expansion_request, &expansion_schedule).unwrap();
        assert_eq!(expansion_receipt.disposition, AuditDisposition::Rejected);
        assert!(expansion_receipt.findings.iter().any(|finding| {
            finding.predicate == "candidate-expansion-limit" && !finding.passed
        }));
    }

    #[test]
    fn rejects_terminal_reserve_violation() {
        let mut request_value: Value = serde_json::from_slice(&h2_request()).unwrap();
        request_value["battery"]["terminal_reserve_quanta"] = serde_json::json!(1);
        let request = serde_json::to_vec(&request_value).unwrap();
        let schedule = h2_schedule(&request);
        let receipt = audit_schedule_bytes(&request, &schedule).unwrap();
        assert_eq!(receipt.disposition, AuditDisposition::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "terminal-reserve" && !finding.passed
        }));
    }

    #[test]
    fn emits_rejected_receipt_with_null_objectives_for_missing_or_extra_intervals() {
        let request = h2_request();
        let baseline: Value = serde_json::from_slice(&h2_schedule(&request)).unwrap();

        let mut missing = baseline.clone();
        missing["interval_dispatches"].as_array_mut().unwrap().pop();
        let missing_receipt = audit_schedule_bytes(
            &request,
            &serde_json::to_vec(&missing).unwrap(),
        )
        .unwrap();
        assert_eq!(missing_receipt.disposition, AuditDisposition::Rejected);
        assert!(missing_receipt
            .recomputed_objectives
            .essential_unserved_quanta
            .is_none());
        assert!(missing_receipt.findings.iter().any(|finding| {
            finding.predicate == "schedule-interval-count" && !finding.passed
        }));

        let mut extra = baseline;
        let duplicated_step = extra["interval_dispatches"][0].clone();
        extra["interval_dispatches"]
            .as_array_mut()
            .unwrap()
            .push(duplicated_step);
        let extra_receipt = audit_schedule_bytes(
            &request,
            &serde_json::to_vec(&extra).unwrap(),
        )
        .unwrap();
        assert_eq!(extra_receipt.disposition, AuditDisposition::Rejected);
        assert!(extra_receipt
            .recomputed_objectives
            .terminal_soc_quanta
            .is_none());
    }

    #[test]
    fn rejects_duplicate_json_keys_unknown_fields_and_invalid_request_sha() {
        assert!(parse_unique_json(br#"{"a":1,"a":2}"#).is_err());
        assert!(parse_unique_json(br#"{"outer":[{"a":1,"a":2}]}"#).is_err());

        let request = h2_request();
        let duplicate_schedule = br#"{"schema_version":"x","schema_version":"y"}"#;
        assert!(audit_schedule_bytes(&request, duplicate_schedule).is_err());

        let mut schedule: Value = serde_json::from_slice(&h2_schedule(&request)).unwrap();
        schedule["unknown_field"] = serde_json::json!(true);
        assert!(audit_schedule_bytes(&request, &serde_json::to_vec(&schedule).unwrap()).is_err());
    }
}
