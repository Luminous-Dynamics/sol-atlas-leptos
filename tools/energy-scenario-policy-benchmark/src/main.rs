#![forbid(unsafe_code)]
//! Receipt-bound exact rational scenario outcome and tail-risk accounting.
//!
//! This tool aggregates independently emitted schedule and service-floor
//! receipts. It does not optimize policies, validate forecast calibration, or
//! establish that scenario probabilities represent the real world.

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const MANIFEST_SCHEMA: &str = "energy-scenario-outcome-panel-v0.1";
const REPORT_SCHEMA: &str = "energy-scenario-outcome-report-v0.1";
const AUDIT_RECEIPT_SCHEMA: &str = "energy-dispatch-schedule-audit-receipt-v0.1";
const AUDITOR_ID: &str = "energy-dispatch-schedule-auditor";
const SERVICE_FLOOR_RECEIPT_SCHEMA: &str = "energy-service-floor-receipt-v0.1";
const MAX_SCENARIOS: usize = 256;
const MAX_PANEL_BYTES: usize = 1_048_576;
const MAX_EVIDENCE_BYTES: usize = 2_097_152;
const MAX_PROBABILITY_COMPONENT: u32 = 1_000_000_000;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
struct Rational {
    numerator: u128,
    denominator: u128,
}

impl Rational {
    fn new(numerator: u128, denominator: u128) -> Result<Self, String> {
        if denominator == 0 {
            return Err("rational denominator must be positive".to_owned());
        }
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    fn is_zero(self) -> bool {
        self.numerator == 0
    }

    fn add(self, other: Self) -> Result<Self, String> {
        let common = gcd(self.denominator, other.denominator);
        let left_factor = other.denominator / common;
        let right_factor = self.denominator / common;
        let left = self
            .numerator
            .checked_mul(left_factor)
            .ok_or_else(|| "rational addition numerator overflow".to_owned())?;
        let right = other
            .numerator
            .checked_mul(right_factor)
            .ok_or_else(|| "rational addition numerator overflow".to_owned())?;
        let numerator = left
            .checked_add(right)
            .ok_or_else(|| "rational addition numerator overflow".to_owned())?;
        let denominator = self
            .denominator
            .checked_mul(left_factor)
            .ok_or_else(|| "rational addition denominator overflow".to_owned())?;
        Self::new(numerator, denominator)
    }

    fn sub(self, other: Self) -> Result<Self, String> {
        match self.cmp_exact(other)? {
            Ordering::Less => Err("rational subtraction would become negative".to_owned()),
            Ordering::Equal => Ok(Self::zero()),
            Ordering::Greater => {
                let common = gcd(self.denominator, other.denominator);
                let left_factor = other.denominator / common;
                let right_factor = self.denominator / common;
                let left = self
                    .numerator
                    .checked_mul(left_factor)
                    .ok_or_else(|| "rational subtraction numerator overflow".to_owned())?;
                let right = other
                    .numerator
                    .checked_mul(right_factor)
                    .ok_or_else(|| "rational subtraction numerator overflow".to_owned())?;
                let denominator = self
                    .denominator
                    .checked_mul(left_factor)
                    .ok_or_else(|| "rational subtraction denominator overflow".to_owned())?;
                Self::new(left - right, denominator)
            }
        }
    }

    fn mul(self, other: Self) -> Result<Self, String> {
        let cross_left = gcd(self.numerator, other.denominator);
        let cross_right = gcd(other.numerator, self.denominator);
        let left_num = self.numerator / cross_left;
        let right_num = other.numerator / cross_right;
        let left_den = self.denominator / cross_right;
        let right_den = other.denominator / cross_left;
        let numerator = left_num
            .checked_mul(right_num)
            .ok_or_else(|| "rational multiplication numerator overflow".to_owned())?;
        let denominator = left_den
            .checked_mul(right_den)
            .ok_or_else(|| "rational multiplication denominator overflow".to_owned())?;
        Self::new(numerator, denominator)
    }

    fn mul_integer(self, value: u64) -> Result<Self, String> {
        self.mul(Self::new(u128::from(value), 1)?)
    }

    fn div(self, other: Self) -> Result<Self, String> {
        if other.numerator == 0 {
            return Err("division by zero rational".to_owned());
        }
        self.mul(Self::new(other.denominator, other.numerator)?)
    }

    fn cmp_exact(self, other: Self) -> Result<Ordering, String> {
        let left = self
            .numerator
            .checked_mul(other.denominator)
            .ok_or_else(|| "rational comparison overflow".to_owned())?;
        let right = other
            .numerator
            .checked_mul(self.denominator)
            .ok_or_else(|| "rational comparison overflow".to_owned())?;
        Ok(left.cmp(&right))
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioSpec {
    scenario_id: String,
    probability_numerator: u32,
    probability_denominator: u32,
    request_path: String,
    schedule_path: String,
    schedule_audit_receipt_path: String,
    service_floor_receipt_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelManifest {
    schema_version: String,
    panel_id: String,
    service_floor_policy_path: String,
    worst_tail_mass_bps: u16,
    scenarios: Vec<ScenarioSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceFloorPolicy {
    schema_version: String,
    policy_id: String,
    minimum_essential_service_bps_per_interval: u16,
    max_total_essential_unserved_quanta: u64,
    max_consecutive_below_floor_intervals: u16,
    minimum_terminal_soc_quanta: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum AuditDisposition {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AuditFinding {
    predicate: String,
    interval_index: Option<usize>,
    interval_id: Option<String>,
    passed: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RecomputedObjectives {
    essential_unserved_quanta: Option<i64>,
    flexible_unserved_quanta: Option<i64>,
    operating_cost_units: Option<i64>,
    grid_carbon_grams: Option<i64>,
    curtailed_pv_quanta: Option<i64>,
    terminal_soc_quanta: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AuditReceipt {
    receipt_schema: String,
    auditor_id: String,
    auditor_version: String,
    auditor_binary_sha256: String,
    source_schema_version: String,
    schedule_schema_version: String,
    case_id: String,
    synthetic_source: bool,
    max_candidate_expansions: u64,
    request_sha256: String,
    schedule_sha256: String,
    interval_count: usize,
    disposition: AuditDisposition,
    findings: Vec<AuditFinding>,
    recomputed_objectives: RecomputedObjectives,
    receipt_sha256: String,
    claim_ceiling: String,
}

#[derive(Serialize)]
struct AuditReceiptSubject<'a> {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ServiceFloorDisposition {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ServiceFloorFinding {
    predicate: String,
    interval_index: Option<usize>,
    interval_id: Option<String>,
    passed: bool,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ServiceFloorReceipt {
    receipt_schema: String,
    policy_id: String,
    policy_sha256: String,
    request_sha256: String,
    schedule_sha256: String,
    schedule_audit_receipt_sha256: String,
    schedule_auditor_binary_sha256: String,
    service_floor_auditor_binary_sha256: String,
    disposition: ServiceFloorDisposition,
    findings: Vec<ServiceFloorFinding>,
    receipt_sha256: String,
    claim_ceiling: String,
}

#[derive(Serialize)]
struct ServiceFloorReceiptSubject<'a> {
    receipt_schema: &'a str,
    policy_id: &'a str,
    policy_sha256: &'a str,
    request_sha256: &'a str,
    schedule_sha256: &'a str,
    schedule_audit_receipt_sha256: &'a str,
    schedule_auditor_binary_sha256: &'a str,
    service_floor_auditor_binary_sha256: &'a str,
    disposition: ServiceFloorDisposition,
    findings: &'a [ServiceFloorFinding],
    claim_ceiling: &'a str,
}

#[derive(Clone, Serialize)]
struct ScenarioOutcome {
    scenario_id: String,
    probability: Rational,
    request_sha256: String,
    schedule_sha256: String,
    schedule_audit_receipt_sha256: String,
    service_floor_receipt_sha256: String,
    service_floor_disposition: ServiceFloorDisposition,
    essential_unserved_quanta: u64,
    flexible_unserved_quanta: u64,
    operating_cost_units: u64,
    grid_carbon_grams: u64,
    curtailed_pv_quanta: u64,
    terminal_soc_quanta: u64,
}

#[derive(Clone, Serialize)]
struct MetricSummary {
    expected: Rational,
    worst_case: u64,
    worst_tail_cvar: Rational,
}

#[derive(Clone, Serialize)]
struct OutcomeMetrics {
    essential_unserved_quanta: MetricSummary,
    flexible_unserved_quanta: MetricSummary,
    operating_cost_units: MetricSummary,
    grid_carbon_grams: MetricSummary,
    curtailed_pv_quanta: MetricSummary,
    terminal_soc_quanta: MetricSummary,
    service_floor_failure_probability: Rational,
    service_floor_failed_scenario_count: usize,
    decision_ready_scenario_count: usize,
    decision_ready_probability: Rational,
}

#[derive(Serialize)]
struct ScenarioOutcomeReportSubject<'a> {
    report_schema: &'a str,
    panel_id: &'a str,
    panel_manifest_sha256: &'a str,
    model_sha256: &'a str,
    benchmark_binary_sha256: &'a str,
    service_floor_policy_id: &'a str,
    service_floor_policy_sha256: &'a str,
    worst_tail_mass_bps: u16,
    probability_total: Rational,
    scenarios: &'a [ScenarioOutcome],
    metrics: &'a OutcomeMetrics,
    claim_ceiling: &'a str,
}

#[derive(Serialize)]
struct ScenarioOutcomeReport {
    report_schema: String,
    panel_id: String,
    panel_manifest_sha256: String,
    model_sha256: String,
    benchmark_binary_sha256: String,
    service_floor_policy_id: String,
    service_floor_policy: ServiceFloorPolicy,
    service_floor_policy_sha256: String,
    worst_tail_mass_bps: u16,
    probability_total: Rational,
    scenarios: Vec<ScenarioOutcome>,
    metrics: OutcomeMetrics,
    report_sha256: String,
    claim_ceiling: String,
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
        formatter.write_str("a JSON value with unique object keys")
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
            .ok_or_else(|| E::custom("JSON numbers must be finite"))
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
                    "duplicate JSON object key {key:?}"
                )));
            }
            let value = object.next_value_seed(StrictJsonSeed)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn parse_unique_json(bytes: &[u8], label: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictJsonSeed
        .deserialize(&mut deserializer)
        .map_err(|error| format!("strict JSON parse for {label}: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("trailing JSON content in {label}: {error}"))?;
    Ok(value)
}

fn parse_typed<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T, String> {
    let value = parse_unique_json(bytes, label)?;
    serde_json::from_value(value).map_err(|error| format!("decode {label}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
}

fn read_bounded(path: &Path, limit: usize, label: &str) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|error| format!("open {label}: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("inspect {label}: {error}"))?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(format!("{label} is not a regular file within {limit} bytes"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {label}: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded the {limit} byte limit while reading"));
    }
    Ok(bytes)
}

fn resolve_under_root(root: &Path, supplied: &str, label: &str) -> Result<PathBuf, String> {
    let relative = Path::new(supplied);
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("{label} must be a non-empty relative path"));
    }
    let canonical = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("resolve {label}: {error}"))?;
    if !canonical.starts_with(root) {
        return Err(format!("{label} resolves outside the manifest directory"));
    }
    Ok(canonical)
}

fn current_binary_sha256() -> Result<String, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve benchmark executable: {error}"))?;
    let bytes = fs::read(executable)
        .map_err(|error| format!("read benchmark executable: {error}"))?;
    Ok(sha256(&bytes))
}

fn audit_receipt_subject(receipt: &AuditReceipt) -> AuditReceiptSubject<'_> {
    AuditReceiptSubject {
        receipt_schema: &receipt.receipt_schema,
        auditor_id: &receipt.auditor_id,
        auditor_version: &receipt.auditor_version,
        auditor_binary_sha256: &receipt.auditor_binary_sha256,
        source_schema_version: &receipt.source_schema_version,
        schedule_schema_version: &receipt.schedule_schema_version,
        case_id: &receipt.case_id,
        synthetic_source: receipt.synthetic_source,
        max_candidate_expansions: receipt.max_candidate_expansions,
        request_sha256: &receipt.request_sha256,
        schedule_sha256: &receipt.schedule_sha256,
        interval_count: receipt.interval_count,
        disposition: receipt.disposition,
        findings: &receipt.findings,
        recomputed_objectives: &receipt.recomputed_objectives,
        claim_ceiling: &receipt.claim_ceiling,
    }
}

fn service_floor_receipt_subject(
    receipt: &ServiceFloorReceipt,
) -> ServiceFloorReceiptSubject<'_> {
    ServiceFloorReceiptSubject {
        receipt_schema: &receipt.receipt_schema,
        policy_id: &receipt.policy_id,
        policy_sha256: &receipt.policy_sha256,
        request_sha256: &receipt.request_sha256,
        schedule_sha256: &receipt.schedule_sha256,
        schedule_audit_receipt_sha256: &receipt.schedule_audit_receipt_sha256,
        schedule_auditor_binary_sha256: &receipt.schedule_auditor_binary_sha256,
        service_floor_auditor_binary_sha256: &receipt.service_floor_auditor_binary_sha256,
        disposition: receipt.disposition,
        findings: &receipt.findings,
        claim_ceiling: &receipt.claim_ceiling,
    }
}

fn verify_receipts(
    request_bytes: &[u8],
    request: &Value,
    schedule_bytes: &[u8],
    audit_receipt_bytes: &[u8],
    service_floor_receipt_bytes: &[u8],
    policy_sha256: &str,
    policy_id: &str,
) -> Result<(AuditReceipt, ServiceFloorReceipt), String> {
    let audit: AuditReceipt = parse_typed(audit_receipt_bytes, "schedule audit receipt")?;
    let service: ServiceFloorReceipt =
        parse_typed(service_floor_receipt_bytes, "service-floor receipt")?;
    let request_sha256 = sha256(request_bytes);
    let schedule_sha256 = sha256(schedule_bytes);
    let case_id = request["case_id"]
        .as_str()
        .ok_or_else(|| "request case_id is missing".to_owned())?;
    let source_schema = request["schema_version"]
        .as_str()
        .ok_or_else(|| "request schema_version is missing".to_owned())?;
    let schedule_value = parse_unique_json(schedule_bytes, "schedule")?;
    let schedule_case_id = schedule_value["case_id"]
        .as_str()
        .ok_or_else(|| "schedule case_id is missing".to_owned())?;
    let schedule_schema = schedule_value["schema_version"]
        .as_str()
        .ok_or_else(|| "schedule schema_version is missing".to_owned())?;
    let interval_count = schedule_value["interval_dispatches"]
        .as_array()
        .ok_or_else(|| "schedule interval_dispatches is not an array".to_owned())?
        .len();
    let synthetic = request["synthetic"]
        .as_bool()
        .ok_or_else(|| "request synthetic field is missing".to_owned())?;
    let max_candidate_expansions = request["max_candidate_expansions"]
        .as_u64()
        .ok_or_else(|| "request max_candidate_expansions is missing".to_owned())?;

    if !valid_sha256(&audit.auditor_binary_sha256)
        || audit.receipt_schema != AUDIT_RECEIPT_SCHEMA
        || audit.auditor_id != AUDITOR_ID
        || audit.auditor_version.trim().is_empty()
        || audit.source_schema_version != source_schema
        || audit.schedule_schema_version != schedule_schema
        || audit.case_id != case_id
        || schedule_case_id != case_id
        || audit.synthetic_source != synthetic
        || audit.max_candidate_expansions != max_candidate_expansions
        || audit.request_sha256 != request_sha256
        || audit.schedule_sha256 != schedule_sha256
        || schedule_value["request_sha256"].as_str() != Some(request_sha256.as_str())
        || audit.interval_count != interval_count
        || audit.disposition != AuditDisposition::Accepted
        || audit.findings.iter().any(|finding| !finding.passed)
        || audit.receipt_sha256 != sha256(
            &serde_json::to_vec(&audit_receipt_subject(&audit))
                .map_err(|error| format!("serialize audit receipt subject: {error}"))?,
        )
    {
        return Err(format!(
            "schedule audit receipt for {case_id} is not an accepted, exact-subject receipt"
        ));
    }
    let objectives = &audit.recomputed_objectives;
    for (name, value) in [
        ("essential_unserved_quanta", objectives.essential_unserved_quanta),
        ("flexible_unserved_quanta", objectives.flexible_unserved_quanta),
        ("operating_cost_units", objectives.operating_cost_units),
        ("grid_carbon_grams", objectives.grid_carbon_grams),
        ("curtailed_pv_quanta", objectives.curtailed_pv_quanta),
        ("terminal_soc_quanta", objectives.terminal_soc_quanta),
    ] {
        if value.is_none_or(|amount| amount < 0) {
            return Err(format!("accepted audit receipt has invalid {name}"));
        }
    }

    if !valid_sha256(&service.service_floor_auditor_binary_sha256)
        || service.receipt_schema != SERVICE_FLOOR_RECEIPT_SCHEMA
        || service.policy_id != policy_id
        || service.policy_sha256 != policy_sha256
        || service.request_sha256 != request_sha256
        || service.schedule_sha256 != schedule_sha256
        || service.schedule_audit_receipt_sha256 != audit.receipt_sha256
        || service.schedule_auditor_binary_sha256 != audit.auditor_binary_sha256
        || match service.disposition {
            ServiceFloorDisposition::Passed => service.findings.iter().any(|finding| !finding.passed),
            ServiceFloorDisposition::Failed => !service.findings.iter().any(|finding| !finding.passed),
        }
        || service.receipt_sha256
            != sha256(
                &serde_json::to_vec(&service_floor_receipt_subject(&service))
                    .map_err(|error| format!("serialize service-floor receipt subject: {error}"))?,
            )
    {
        return Err(format!(
            "service-floor receipt for {case_id} is not a valid exact-subject receipt"
        ));
    }
    Ok((audit, service))
}

fn model_fingerprint(request: &Value) -> Result<String, String> {
    let intervals = request["intervals"]
        .as_array()
        .ok_or_else(|| "request intervals is not an array".to_owned())?;
    let chronology: Vec<_> = intervals
        .iter()
        .map(|interval| {
            serde_json::json!({
                "interval_id": interval["interval_id"],
                "start_unix_seconds": interval["start_unix_seconds"],
                "duration_seconds": interval["duration_seconds"]
            })
        })
        .collect();
    let model = serde_json::json!({
        "schema_version": request["schema_version"],
        "synthetic": request["synthetic"],
        "evidence_class": request["evidence_class"],
        "energy_quantum_kwh": request["energy_quantum_kwh"],
        "battery": request["battery"],
        "chronology": chronology
    });
    let bytes = serde_json::to_vec(&model)
        .map_err(|error| format!("serialize model fingerprint: {error}"))?;
    Ok(sha256(&bytes))
}

fn validate_probability(spec: &ScenarioSpec) -> Result<Rational, String> {
    if spec.probability_numerator == 0
        || spec.probability_denominator == 0
        || spec.probability_numerator > MAX_PROBABILITY_COMPONENT
        || spec.probability_denominator > MAX_PROBABILITY_COMPONENT
        || spec.probability_numerator > spec.probability_denominator
    {
        return Err(format!("scenario {} has an invalid rational probability", spec.scenario_id));
    }
    Rational::new(
        u128::from(spec.probability_numerator),
        u128::from(spec.probability_denominator),
    )
}

fn metric_summary(
    scenarios: &[ScenarioOutcome],
    tail_mass: Rational,
    value: impl Fn(&ScenarioOutcome) -> u64,
) -> Result<MetricSummary, String> {
    let mut expected = Rational::zero();
    let mut losses = Vec::with_capacity(scenarios.len());
    let mut worst_case = 0_u64;
    for scenario in scenarios {
        let metric = value(scenario);
        worst_case = worst_case.max(metric);
        expected = expected.add(scenario.probability.mul_integer(metric)?)?;
        losses.push((metric, scenario.probability));
    }
    let worst_tail_cvar = cvar_upper_tail(&mut losses, tail_mass)?;
    Ok(MetricSummary {
        expected,
        worst_case,
        worst_tail_cvar,
    })
}

/// Exact discrete upper-tail mean. Probability mass at the boundary is
/// fractionally allocated so the selected worst tail has exactly tail_mass.
fn cvar_upper_tail(
    losses: &mut [(u64, Rational)],
    tail_mass: Rational,
) -> Result<Rational, String> {
    if tail_mass.is_zero() || tail_mass.cmp_exact(Rational::one())? == Ordering::Greater {
        return Err("CVaR tail mass must be in (0, 1]".to_owned());
    }
    losses.sort_by(|left, right| right.0.cmp(&left.0));
    let mut remaining = tail_mass;
    let mut weighted_loss = Rational::zero();
    for (loss, probability) in losses.iter().copied() {
        if remaining.is_zero() {
            break;
        }
        let take = if probability.cmp_exact(remaining)? == Ordering::Less {
            probability
        } else {
            remaining
        };
        weighted_loss = weighted_loss.add(take.mul_integer(loss)?)?;
        remaining = remaining.sub(take)?;
    }
    if !remaining.is_zero() {
        return Err("scenario probability mass is insufficient for CVaR tail".to_owned());
    }
    weighted_loss.div(tail_mass)
}

fn aggregate(
    panel_id: &str,
    panel_manifest_sha256: &str,
    model_sha256: &str,
    policy: &ServiceFloorPolicy,
    policy_sha256: &str,
    tail_mass_bps: u16,
    scenarios: Vec<ScenarioOutcome>,
    benchmark_binary_sha256: String,
) -> Result<ScenarioOutcomeReport, String> {
    let mut probability_total = Rational::zero();
    for scenario in &scenarios {
        probability_total = probability_total.add(scenario.probability)?;
    }
    if probability_total != Rational::one() {
        return Err(format!(
            "scenario probabilities must sum exactly to 1; received {}/{}",
            probability_total.numerator, probability_total.denominator
        ));
    }
    let tail_mass = Rational::new(u128::from(tail_mass_bps), 10_000)?;
    let service_floor_failed: Vec<_> = scenarios
        .iter()
        .filter(|scenario| scenario.service_floor_disposition == ServiceFloorDisposition::Failed)
        .collect();
    let mut service_floor_failure_probability = Rational::zero();
    for scenario in &service_floor_failed {
        service_floor_failure_probability =
            service_floor_failure_probability.add(scenario.probability)?;
    }
    let mut decision_ready_probability = Rational::zero();
    for scenario in &scenarios {
        if scenario.service_floor_disposition == ServiceFloorDisposition::Passed {
            decision_ready_probability =
                decision_ready_probability.add(scenario.probability)?;
        }
    }

    let metrics = OutcomeMetrics {
        essential_unserved_quanta: metric_summary(
            &scenarios,
            tail_mass,
            |scenario| scenario.essential_unserved_quanta,
        )?,
        flexible_unserved_quanta: metric_summary(
            &scenarios,
            tail_mass,
            |scenario| scenario.flexible_unserved_quanta,
        )?,
        operating_cost_units: metric_summary(&scenarios, tail_mass, |scenario| {
            scenario.operating_cost_units
        })?,
        grid_carbon_grams: metric_summary(&scenarios, tail_mass, |scenario| {
            scenario.grid_carbon_grams
        })?,
        curtailed_pv_quanta: metric_summary(&scenarios, tail_mass, |scenario| {
            scenario.curtailed_pv_quanta
        })?,
        terminal_soc_quanta: metric_summary_lower_tail(&scenarios, tail_mass, |scenario| {
            scenario.terminal_soc_quanta
        })?,
        service_floor_failure_probability,
        service_floor_failed_scenario_count: service_floor_failed.len(),
        decision_ready_scenario_count: scenarios.len() - service_floor_failed.len(),
        decision_ready_probability,
    };

    let claim_ceiling = concat!(
        "exact rational expected, worst-case and discrete upper-tail outcome summaries ",
        "under the supplied scenario weights for one common finite physical model; ",
        "weights are not calibrated real-world probabilities; no optimization, EVPI, ",
        "policy optimality, probability calibration, reliability or deployment claim"
    );
    let subject = ScenarioOutcomeReportSubject {
        report_schema: REPORT_SCHEMA,
        panel_id,
        panel_manifest_sha256,
        model_sha256,
        benchmark_binary_sha256: &benchmark_binary_sha256,
        service_floor_policy_id: &policy.policy_id,
        service_floor_policy_sha256: policy_sha256,
        worst_tail_mass_bps: tail_mass_bps,
        probability_total,
        scenarios: &scenarios,
        metrics: &metrics,
        claim_ceiling,
    };
    let report_sha256 = sha256(
        &serde_json::to_vec(&subject)
            .map_err(|error| format!("serialize scenario report subject: {error}"))?,
    );
    Ok(ScenarioOutcomeReport {
        report_schema: REPORT_SCHEMA.to_owned(),
        panel_id: panel_id.to_owned(),
        panel_manifest_sha256: panel_manifest_sha256.to_owned(),
        model_sha256: model_sha256.to_owned(),
        benchmark_binary_sha256,
        service_floor_policy_id: policy.policy_id.clone(),
        service_floor_policy: policy.clone(),
        service_floor_policy_sha256: policy_sha256.to_owned(),
        worst_tail_mass_bps: tail_mass_bps,
        probability_total,
        scenarios,
        metrics,
        report_sha256,
        claim_ceiling: claim_ceiling.to_owned(),
    })
}

fn load_and_aggregate(
    manifest_path: &Path,
) -> Result<ScenarioOutcomeReport, String> {
    let manifest_path = manifest_path
        .canonicalize()
        .map_err(|error| format!("resolve manifest path: {error}"))?;
    let manifest_bytes = read_bounded(&manifest_path, MAX_PANEL_BYTES, "scenario panel manifest")?;
    let manifest: PanelManifest = parse_typed(&manifest_bytes, "scenario panel manifest")?;
    if manifest.schema_version != MANIFEST_SCHEMA
        || !safe_identifier(&manifest.panel_id)
        || !(2..=MAX_SCENARIOS).contains(&manifest.scenarios.len())
        || !(1..=10_000).contains(&manifest.worst_tail_mass_bps)
    {
        return Err("scenario panel manifest schema, ID, count or tail mass is invalid".to_owned());
    }

    let root = manifest_path
        .parent()
        .ok_or_else(|| "canonical manifest has no parent".to_owned())?
        .to_path_buf();
    let policy_path =
        resolve_under_root(&root, &manifest.service_floor_policy_path, "service_floor_policy_path")?;
    let policy_bytes = read_bounded(&policy_path, MAX_EVIDENCE_BYTES, "service-floor policy")?;
    let policy: ServiceFloorPolicy = parse_typed(&policy_bytes, "service-floor policy")?;
    if policy.schema_version != "energy-service-floor-policy-v0.1"
        || !safe_identifier(&policy.policy_id)
        || policy.minimum_essential_service_bps_per_interval > 10_000
    {
        return Err("service-floor policy schema or values are invalid".to_owned());
    }
    let policy_sha256 = sha256(&policy_bytes);

    let mut scenario_ids = BTreeSet::new();
    let mut model_identity: Option<String> = None;
    let mut scenarios = Vec::with_capacity(manifest.scenarios.len());
    let mut probability_total = Rational::zero();
    for spec in manifest.scenarios {
        if !safe_identifier(&spec.scenario_id) || !scenario_ids.insert(spec.scenario_id.clone()) {
            return Err("scenario IDs must be safe and unique".to_owned());
        }
        let probability = validate_probability(&spec)?;
        probability_total = probability_total.add(probability)?;

        let request_path = resolve_under_root(&root, &spec.request_path, "request_path")?;
        let schedule_path = resolve_under_root(&root, &spec.schedule_path, "schedule_path")?;
        let audit_path = resolve_under_root(
            &root,
            &spec.schedule_audit_receipt_path,
            "schedule_audit_receipt_path",
        )?;
        let service_path = resolve_under_root(
            &root,
            &spec.service_floor_receipt_path,
            "service_floor_receipt_path",
        )?;

        let request_bytes = read_bounded(&request_path, MAX_EVIDENCE_BYTES, "scenario request")?;
        let request_value = parse_unique_json(&request_bytes, "scenario request")?;
        let schedule_bytes = read_bounded(&schedule_path, MAX_EVIDENCE_BYTES, "scenario schedule")?;
        let audit_receipt_bytes =
            read_bounded(&audit_path, MAX_EVIDENCE_BYTES, "schedule audit receipt")?;
        let service_floor_receipt_bytes =
            read_bounded(&service_path, MAX_EVIDENCE_BYTES, "service-floor receipt")?;

        let request_case_id = request_value["case_id"]
            .as_str()
            .ok_or_else(|| "scenario request case_id is missing".to_owned())?;
        if request_case_id != spec.scenario_id {
            return Err(format!(
                "scenario ID {} does not match request case_id {request_case_id}",
                spec.scenario_id
            ));
        }
        let model = model_fingerprint(&request_value)?;
        if let Some(existing) = &model_identity {
            if existing != &model {
                return Err("scenario requests do not share one comparable physical model".to_owned());
            }
        } else {
            model_identity = Some(model.clone());
        }

        let (audit_receipt, service_floor_receipt) = verify_receipts(
            &request_bytes,
            &request_value,
            &schedule_bytes,
            &audit_receipt_bytes,
            &service_floor_receipt_bytes,
            &policy_sha256,
            &policy.policy_id,
        )?;
        let objective = &audit_receipt.recomputed_objectives;
        let to_u64 = |name: &str, value: Option<i64>| -> Result<u64, String> {
            value
                .and_then(|value| u64::try_from(value).ok())
                .ok_or_else(|| format!("accepted audit objective {name} is invalid"))
        };
        let outcome = ScenarioOutcome {
            scenario_id: spec.scenario_id,
            probability,
            request_sha256: audit_receipt.request_sha256.clone(),
            schedule_sha256: audit_receipt.schedule_sha256.clone(),
            schedule_audit_receipt_sha256: audit_receipt.receipt_sha256.clone(),
            service_floor_receipt_sha256: service_floor_receipt.receipt_sha256.clone(),
            service_floor_disposition: service_floor_receipt.disposition,
            essential_unserved_quanta: to_u64(
                "essential_unserved_quanta",
                objective.essential_unserved_quanta,
            )?,
            flexible_unserved_quanta: to_u64(
                "flexible_unserved_quanta",
                objective.flexible_unserved_quanta,
            )?,
            operating_cost_units: to_u64("operating_cost_units", objective.operating_cost_units)?,
            grid_carbon_grams: to_u64("grid_carbon_grams", objective.grid_carbon_grams)?,
            curtailed_pv_quanta: to_u64("curtailed_pv_quanta", objective.curtailed_pv_quanta)?,
            terminal_soc_quanta: to_u64("terminal_soc_quanta", objective.terminal_soc_quanta)?,
        };
        scenarios.push(outcome);
    }
    if probability_total != Rational::one() {
        return Err("scenario probabilities must sum exactly to 1".to_owned());
    }
    let model_sha256 = model_identity
        .ok_or_else(|| "scenario panel has no model identity".to_owned())?;
    let report = aggregate(
        &manifest.panel_id,
        &sha256(&manifest_bytes),
        &model_sha256,
        &policy,
        &policy_sha256,
        manifest.worst_tail_mass_bps,
        scenarios,
        current_binary_sha256()?,
    )?;
    Ok(report)
}

fn write_report(path: &Path, report: &ScenarioOutcomeReport) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("serialize scenario outcome report: {error}"))?;
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create report output directory: {error}"))?;
    }
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("write temporary scenario outcome report: {error}"))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("publish scenario outcome report: {error}"))
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err(format!(
            "usage: {} <scenario-panel.json> <outcome-report.json>",
            args.first()
                .map(|value| value.to_string_lossy())
                .unwrap_or_else(|| "energy-scenario-policy-benchmark".into())
        ));
    }
    let report = load_and_aggregate(Path::new(&args[1]))?;
    write_report(Path::new(&args[2]), &report)?;
    println!("panel_id={}", report.panel_id);
    println!("scenario_count={}", report.scenarios.len());
    println!("probability_total={}/{}", report.probability_total.numerator, report.probability_total.denominator);
    println!("decision_ready_probability={}/{}", report.metrics.decision_ready_probability.numerator, report.metrics.decision_ready_probability.denominator);
    println!("service_floor_failure_probability={}/{}", report.metrics.service_floor_failure_probability.numerator, report.metrics.service_floor_failure_probability.denominator);
    println!("report_sha256={}", report.report_sha256);
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("scenario outcome accounting failed closed: {error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_arithmetic_reduces_and_preserves_exactness() {
        let three_tenths = Rational::new(3, 10).unwrap();
        let seven_tenths = Rational::new(7, 10).unwrap();
        assert_eq!(three_tenths.add(seven_tenths).unwrap(), Rational::one());
        assert_eq!(Rational::new(6, 20).unwrap(), three_tenths);
        assert_eq!(
            three_tenths.mul_integer(100).unwrap(),
            Rational::new(30, 1).unwrap()
        );
    }

    #[test]
    fn rational_probability_mass_must_be_exactly_one() {
        let total = Rational::new(1, 3)
            .unwrap()
            .add(Rational::new(2, 3).unwrap())
            .unwrap();
        assert_eq!(total, Rational::one());
        let underweight = Rational::new(1, 3)
            .unwrap()
            .add(Rational::new(1, 3).unwrap())
            .unwrap();
        assert_ne!(underweight, Rational::one());
    }

    #[test]
    fn cvar_fractionally_allocates_probability_at_the_tail_boundary() {
        let mut losses = vec![
            (100_u64, Rational::new(3, 10).unwrap()),
            (0_u64, Rational::new(7, 10).unwrap()),
        ];
        let cvar = cvar_upper_tail(&mut losses, Rational::new(1, 2).unwrap()).unwrap();
        assert_eq!(cvar, Rational::new(60, 1).unwrap());
    }

    #[test]
    fn cvar_full_tail_equals_expected_loss() {
        let mut losses = vec![
            (10_u64, Rational::new(1, 4).unwrap()),
            (20_u64, Rational::new(3, 4).unwrap()),
        ];
        let cvar = cvar_upper_tail(&mut losses, Rational::one()).unwrap();
        assert_eq!(cvar, Rational::new(35, 2).unwrap());
    }

    #[test]
    fn rational_overflow_fails_closed() {
        let huge = Rational::new(u128::MAX, 1).unwrap();
        assert!(huge.add(Rational::one()).is_err());
    }
}
