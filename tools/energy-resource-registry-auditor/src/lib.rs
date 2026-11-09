//! Strict, standalone conformance checks for the Energy Resource Source Registry v0.1.
//!
//! This crate validates the registry contract and its relationship to the committed
//! JSON Schema. It does not fetch sources, authenticate publishers, decide licence
//! sufficiency, validate resource science, or admit source data into Sol-Atlas.

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

pub const AUDITOR_ID: &str = "energy-resource-registry-auditor";
pub const AUDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REGISTRY_SCHEMA_VERSION: &str = "0.1.0";
pub const REGISTRY_STATUS: &str = "discovery_only_not_admitted";
pub const RECEIPT_SCHEMA: &str = "energy-source-registry-audit-receipt-v0.1";
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;

const TOP_LEVEL_FIELDS: &[&str] = &[
    "schema_version",
    "as_of",
    "registry_status",
    "claim_ceiling",
    "entries",
];

const ENTRY_FIELDS: &[&str] = &[
    "source_id",
    "source_name",
    "provider",
    "source_url",
    "source_type",
    "resource_families",
    "evidence_basis",
    "availability_state",
    "source_version",
    "spatial_scope",
    "spatial_resolution",
    "temporal_scope",
    "temporal_resolution",
    "variables",
    "license_status",
    "claim_ceiling",
    "limitations",
    "admission_requirements",
    "raw_snapshot_ingested",
    "snapshot_digest",
];

const SOURCE_VERSION_FIELDS: &[&str] = &[
    "label",
    "release_or_publication_date",
    "data_period",
    "metadata_updated",
    "notes",
];

const LICENSE_STATUS_FIELDS: &[&str] = &["status", "details"];

const SOURCE_TYPE_VALUES: &[&str] = &[
    "dataset",
    "dataset_catalogue",
    "research_publication",
    "government_page",
    "sector_report",
    "portal_status",
];

const RESOURCE_FAMILY_VALUES: &[&str] = &[
    "solar",
    "onshore_wind",
    "offshore_wind",
    "hydropower",
    "geothermal",
    "ocean_current",
    "wave",
    "marine_physics",
    "cross_resource",
];

const EVIDENCE_BASIS_VALUES: &[&str] = &[
    "modeled_resource_atlas",
    "modeled_reanalysis",
    "modeled_analysis_forecast",
    "published_field_observation",
    "government_historical_assessment",
    "government_research_context",
    "sector_policy_report",
    "discovery_catalogue",
    "availability_notice",
    "literature_review",
];

const AVAILABILITY_STATE_VALUES: &[&str] = &[
    "available_pending_snapshot_pin",
    "temporarily_unavailable",
    "context_only",
    "catalogue_only",
    "needs_access_or_measurement",
];

const LICENSE_STATUS_VALUES: &[&str] = &[
    "identified_requires_review",
    "unknown",
    "not_assessed",
    "open_access_catalogue_only",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditError(pub String);

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AuditError {}

fn error(message: impl Into<String>) -> AuditError {
    AuditError(message.into())
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

// serde_json::Value normally keeps the last occurrence of a duplicate object
// member. This visitor rejects duplicates before conversion to Value.
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
        formatter.write_str("a JSON value with unique object member names")
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
                    "duplicate JSON object member {key:?}"
                )));
            }
            let value = object.next_value_seed(StrictJsonSeed)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

/// Strictly parse exactly one JSON value. Duplicate object members and
/// trailing non-whitespace content are rejected.
pub fn parse_unique_json(bytes: &[u8]) -> Result<Value, AuditError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(error(format!(
            "input exceeds maximum size of {MAX_INPUT_BYTES} bytes"
        )));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictJsonSeed
        .deserialize(&mut deserializer)
        .map_err(|cause| error(format!("strict JSON parse failed: {cause}")))?;
    deserializer
        .end()
        .map_err(|cause| error(format!("trailing JSON content: {cause}")))?;
    Ok(value)
}

fn string_set(value: Option<&Value>) -> Option<BTreeSet<String>> {
    let values = value?.as_array()?;
    let mut output = BTreeSet::new();
    for item in values {
        output.insert(item.as_str()?.to_owned());
    }
    Some(output)
}

fn expected_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn object_keys(value: Option<&Value>) -> Option<BTreeSet<String>> {
    let object = value?.as_object()?;
    Some(object.keys().cloned().collect())
}

fn schema_has_exact_fields(
    schema: &Value,
    object_path: &[&str],
    expected_fields: &[&str],
    required_path: &[&str],
    predicate: &str,
    findings: &mut Vec<AuditFinding>,
) {
    let mut object = schema;
    for part in object_path {
        let Some(next) = object.get(*part) else {
            findings.push(AuditFinding::fail(
                predicate,
                format!("missing schema object path component {part}"),
            ));
            return;
        };
        object = next;
    }

    let expected = expected_set(expected_fields);
    match object_keys(Some(object)) {
        Some(actual) if actual == expected => {}
        Some(actual) => findings.push(AuditFinding::fail(
            predicate,
            format!("schema property keys differ; expected {expected:?}, found {actual:?}"),
        )),
        None => findings.push(AuditFinding::fail(
            predicate,
            "schema property container must be an object",
        )),
    }

    let mut required = schema;
    for part in required_path {
        let Some(next) = required.get(*part) else {
            findings.push(AuditFinding::fail(
                predicate,
                format!("missing required-array path component {part}"),
            ));
            return;
        };
        required = next;
    }
    match string_set(Some(required)) {
        Some(actual) if actual == expected => {
            if required
                .as_array()
                .is_none_or(|values| values.len() != expected.len())
            {
                findings.push(AuditFinding::fail(
                    predicate,
                    "required array contains duplicate or non-string values",
                ));
            }
        }
        Some(actual) => findings.push(AuditFinding::fail(
            predicate,
            format!("required keys differ; expected {expected:?}, found {actual:?}"),
        )),
        None => findings.push(AuditFinding::fail(
            predicate,
            "required must be an array of strings",
        )),
    }
}

fn check_enum(
    schema: &Value,
    property_path: &[&str],
    expected_values: &[&str],
    predicate: &str,
    findings: &mut Vec<AuditFinding>,
) {
    let mut value = schema;
    for part in property_path {
        let Some(next) = value.get(*part) else {
            findings.push(AuditFinding::fail(
                predicate,
                format!("missing enum path component {part}"),
            ));
            return;
        };
        value = next;
    }
    let expected = expected_set(expected_values);
    match string_set(Some(value)) {
        Some(actual) if actual == expected => {
            if value
                .as_array()
                .is_none_or(|values| values.len() != expected.len())
            {
                findings.push(AuditFinding::fail(
                    predicate,
                    "enum contains duplicate or non-string values",
                ));
            }
        }
        Some(actual) => findings.push(AuditFinding::fail(
            predicate,
            format!("enum differs; expected {expected:?}, found {actual:?}"),
        )),
        None => findings.push(AuditFinding::fail(
            predicate,
            "enum must be an array of strings",
        )),
    }
}

fn check_schema_contract(schema: &Value, findings: &mut Vec<AuditFinding>) {
    if schema.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        findings.push(AuditFinding::fail(
            "schema_draft_2020_12",
            "schema must declare the JSON Schema Draft 2020-12 meta-schema",
        ));
    }
    if schema.get("type").and_then(Value::as_str) != Some("object")
        || schema.get("additionalProperties").and_then(Value::as_bool) != Some(false)
    {
        findings.push(AuditFinding::fail(
            "schema_root_closed_object",
            "root schema must be an object with additionalProperties=false",
        ));
    }
    if schema
        .pointer("/properties/schema_version/const")
        .and_then(Value::as_str)
        != Some(REGISTRY_SCHEMA_VERSION)
    {
        findings.push(AuditFinding::fail(
            "schema_version_const",
            "schema version const must equal 0.1.0",
        ));
    }
    if schema
        .pointer("/properties/registry_status/const")
        .and_then(Value::as_str)
        != Some(REGISTRY_STATUS)
    {
        findings.push(AuditFinding::fail(
            "schema_registry_status_const",
            "registry status const must preserve discovery_only_not_admitted",
        ));
    }
    if schema
        .pointer("/properties/as_of/format")
        .and_then(Value::as_str)
        != Some("date")
    {
        findings.push(AuditFinding::fail(
            "schema_as_of_date_format",
            "as_of must use JSON Schema date format",
        ));
    }
    if schema
        .pointer("/properties/entries/items/$ref")
        .and_then(Value::as_str)
        != Some("#/$defs/source_entry")
    {
        findings.push(AuditFinding::fail(
            "schema_entry_reference",
            "entries.items must reference #/$defs/source_entry",
        ));
    }
    if schema
        .pointer("/properties/entries/minItems")
        .and_then(Value::as_u64)
        != Some(1)
    {
        findings.push(AuditFinding::fail(
            "schema_entries_minimum",
            "entries must require at least one source record",
        ));
    }
    if schema
        .pointer("/$defs/source_entry/properties/resource_families/uniqueItems")
        .and_then(Value::as_bool)
        != Some(true)
    {
        findings.push(AuditFinding::fail(
            "schema_resource_families_unique",
            "resource_families must declare uniqueItems=true",
        ));
    }

    schema_has_exact_fields(
        schema,
        &["properties"],
        TOP_LEVEL_FIELDS,
        &["required"],
        "schema_top_level_fields",
        findings,
    );
    schema_has_exact_fields(
        schema,
        &["$defs", "source_entry", "properties"],
        ENTRY_FIELDS,
        &["$defs", "source_entry", "required"],
        "schema_source_entry_fields",
        findings,
    );
    schema_has_exact_fields(
        schema,
        &[
            "$defs",
            "source_entry",
            "properties",
            "source_version",
            "properties",
        ],
        SOURCE_VERSION_FIELDS,
        &[
            "$defs",
            "source_entry",
            "properties",
            "source_version",
            "required",
        ],
        "schema_source_version_fields",
        findings,
    );
    schema_has_exact_fields(
        schema,
        &[
            "$defs",
            "source_entry",
            "properties",
            "license_status",
            "properties",
        ],
        LICENSE_STATUS_FIELDS,
        &[
            "$defs",
            "source_entry",
            "properties",
            "license_status",
            "required",
        ],
        "schema_license_status_fields",
        findings,
    );

    for (name, expected) in [
        ("source_type", SOURCE_TYPE_VALUES),
        ("evidence_basis", EVIDENCE_BASIS_VALUES),
        ("availability_state", AVAILABILITY_STATE_VALUES),
    ] {
        check_enum(
            schema,
            &["$defs", "source_entry", "properties", name, "enum"],
            expected,
            "schema_enum_values",
            findings,
        );
    }
    check_enum(
        schema,
        &[
            "$defs",
            "source_entry",
            "properties",
            "resource_families",
            "items",
            "enum",
        ],
        RESOURCE_FAMILY_VALUES,
        "schema_enum_values",
        findings,
    );
    check_enum(
        schema,
        &[
            "$defs",
            "source_entry",
            "properties",
            "license_status",
            "properties",
            "status",
            "enum",
        ],
        LICENSE_STATUS_VALUES,
        "schema_enum_values",
        findings,
    );

    if schema
        .pointer("/$defs/source_entry/additionalProperties")
        .and_then(Value::as_bool)
        != Some(false)
        || schema
            .pointer("/$defs/source_entry/properties/source_version/additionalProperties")
            .and_then(Value::as_bool)
            != Some(false)
        || schema
            .pointer("/$defs/source_entry/properties/license_status/additionalProperties")
            .and_then(Value::as_bool)
            != Some(false)
    {
        findings.push(AuditFinding::fail(
            "schema_nested_closed_objects",
            "source_entry and nested objects must reject unknown properties",
        ));
    }
    if schema
        .pointer("/$defs/source_entry/properties/raw_snapshot_ingested/const")
        .and_then(Value::as_bool)
        != Some(false)
        || schema
            .pointer("/$defs/source_entry/properties/snapshot_digest/type")
            .and_then(Value::as_str)
            != Some("null")
    {
        findings.push(AuditFinding::fail(
            "schema_discovery_only_snapshot",
            "v0.1 must require raw_snapshot_ingested=false and snapshot_digest=null",
        ));
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: String,
    as_of: String,
    registry_status: String,
    claim_ceiling: String,
    entries: Vec<SourceEntry>,
}

fn deserialize_required_nullable<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEntry {
    source_id: String,
    source_name: String,
    provider: String,
    source_url: String,
    source_type: SourceType,
    resource_families: Vec<ResourceFamily>,
    evidence_basis: EvidenceBasis,
    availability_state: AvailabilityState,
    source_version: SourceVersion,
    spatial_scope: String,
    spatial_resolution: String,
    temporal_scope: String,
    temporal_resolution: String,
    variables: Vec<String>,
    license_status: LicenseStatus,
    claim_ceiling: String,
    limitations: Vec<String>,
    admission_requirements: Vec<String>,
    raw_snapshot_ingested: bool,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    snapshot_digest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceVersion {
    label: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    release_or_publication_date: Option<String>,
    data_period: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    metadata_updated: Option<String>,
    notes: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LicenseStatus {
    status: LicenseState,
    details: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SourceType {
    Dataset,
    DatasetCatalogue,
    ResearchPublication,
    GovernmentPage,
    SectorReport,
    PortalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResourceFamily {
    Solar,
    OnshoreWind,
    OffshoreWind,
    Hydropower,
    Geothermal,
    OceanCurrent,
    Wave,
    MarinePhysics,
    CrossResource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceBasis {
    ModeledResourceAtlas,
    ModeledReanalysis,
    ModeledAnalysisForecast,
    PublishedFieldObservation,
    GovernmentHistoricalAssessment,
    GovernmentResearchContext,
    SectorPolicyReport,
    DiscoveryCatalogue,
    AvailabilityNotice,
    LiteratureReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AvailabilityState {
    AvailablePendingSnapshotPin,
    TemporarilyUnavailable,
    ContextOnly,
    CatalogueOnly,
    NeedsAccessOrMeasurement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LicenseState {
    IdentifiedRequiresReview,
    Unknown,
    NotAssessed,
    OpenAccessCatalogueOnly,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditFinding {
    pub code: String,
    pub detail: String,
}

impl AuditFinding {
    fn fail(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuditReceipt {
    pub receipt_schema: String,
    pub auditor_id: String,
    pub auditor_version: String,
    pub schema_sha256: String,
    pub registry_sha256: String,
    pub registry_status: String,
    pub entry_count: usize,
    pub source_ids: Vec<String>,
    pub disposition: String,
    pub findings: Vec<AuditFinding>,
    pub claim_ceiling: String,
    pub receipt_sha256: String,
}

#[derive(Serialize)]
struct ReceiptSubject<'a> {
    receipt_schema: &'a str,
    auditor_id: &'a str,
    auditor_version: &'a str,
    schema_sha256: &'a str,
    registry_sha256: &'a str,
    registry_status: &'a str,
    entry_count: usize,
    source_ids: &'a [String],
    disposition: &'a str,
    findings: &'a [AuditFinding],
    claim_ceiling: &'a str,
}

fn is_nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn is_safe_source_id(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_date_precision(value: &str, allow_year_or_month: bool) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    let digits_only =
        |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    match parts.as_slice() {
        [year] if allow_year_or_month => {
            year.len() == 4 && digits_only(year) && year.parse::<u16>().is_ok()
        }
        [year, month] if allow_year_or_month => {
            year.len() == 4
                && digits_only(year)
                && digits_only(month)
                && year.parse::<u16>().is_ok()
                && month
                    .parse::<u8>()
                    .is_ok_and(|month| (1..=12).contains(&month))
        }
        [year, month, day] => {
            if year.len() != 4
                || month.len() != 2
                || day.len() != 2
                || !digits_only(year)
                || !digits_only(month)
                || !digits_only(day)
            {
                return false;
            }
            let Ok(year) = year.parse::<u16>() else {
                return false;
            };
            let Ok(month) = month.parse::<u8>() else {
                return false;
            };
            let Ok(day) = day.parse::<u8>() else {
                return false;
            };
            let maximum_day = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
                2 => 28,
                _ => return false,
            };
            (1..=maximum_day).contains(&day)
        }
        _ => false,
    }
}

fn valid_https_url(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    if rest.is_empty() || rest.chars().any(char::is_whitespace) {
        return false;
    }
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    !authority.is_empty()
        && !authority.starts_with('.')
        && !authority.ends_with('.')
        && !authority.contains('@')
}

fn validate_registry(registry: &Registry, findings: &mut Vec<AuditFinding>) {
    if registry.schema_version != REGISTRY_SCHEMA_VERSION {
        findings.push(AuditFinding::fail(
            "registry_schema_version",
            "registry schema_version must equal 0.1.0",
        ));
    }
    if registry.registry_status != REGISTRY_STATUS {
        findings.push(AuditFinding::fail(
            "registry_discovery_only_status",
            "v0.1 registry_status must remain discovery_only_not_admitted",
        ));
    }
    if !valid_date_precision(&registry.as_of, false) {
        findings.push(AuditFinding::fail(
            "registry_as_of_date",
            "as_of must be a valid YYYY-MM-DD calendar date",
        ));
    }
    if !is_nonempty(&registry.claim_ceiling) {
        findings.push(AuditFinding::fail(
            "registry_claim_ceiling",
            "registry claim_ceiling must not be empty",
        ));
    }
    if registry.entries.is_empty() {
        findings.push(AuditFinding::fail(
            "registry_entries_nonempty",
            "registry must contain at least one source entry",
        ));
    }

    let mut source_ids = BTreeSet::new();
    for entry in &registry.entries {
        if !source_ids.insert(entry.source_id.as_str()) {
            findings.push(AuditFinding::fail(
                "source_id_unique",
                format!("duplicate source_id: {}", entry.source_id),
            ));
        }
        validate_entry(entry, findings);
    }
}

fn validate_entry(entry: &SourceEntry, findings: &mut Vec<AuditFinding>) {
    let id = &entry.source_id;

    if !is_safe_source_id(id) {
        findings.push(AuditFinding::fail(
            "source_id_format",
            format!("{id}: source_id must be lowercase kebab-case ASCII"),
        ));
    }

    for (field, value) in [
        ("source_name", entry.source_name.as_str()),
        ("provider", entry.provider.as_str()),
        ("spatial_scope", entry.spatial_scope.as_str()),
        ("spatial_resolution", entry.spatial_resolution.as_str()),
        ("temporal_scope", entry.temporal_scope.as_str()),
        ("temporal_resolution", entry.temporal_resolution.as_str()),
        ("claim_ceiling", entry.claim_ceiling.as_str()),
        ("source_version.label", entry.source_version.label.as_str()),
        (
            "source_version.data_period",
            entry.source_version.data_period.as_str(),
        ),
        ("source_version.notes", entry.source_version.notes.as_str()),
        (
            "license_status.details",
            entry.license_status.details.as_str(),
        ),
    ] {
        if !is_nonempty(value) {
            findings.push(AuditFinding::fail(
                "required_text_nonempty",
                format!("{id}: {field} must not be empty"),
            ));
        }
    }

    if !valid_https_url(&entry.source_url) {
        findings.push(AuditFinding::fail(
            "source_url_https",
            format!("{id}: source_url must be a non-empty HTTPS locator"),
        ));
    }

    if entry.resource_families.is_empty() {
        findings.push(AuditFinding::fail(
            "resource_families_nonempty",
            format!("{id}: resource_families must not be empty"),
        ));
    }
    let family_count = entry.resource_families.len();
    let family_unique_count: BTreeSet<_> = entry.resource_families.iter().collect();
    if family_count != family_unique_count.len() {
        findings.push(AuditFinding::fail(
            "resource_families_unique",
            format!("{id}: resource_families must not contain duplicates"),
        ));
    }

    if entry.variables.iter().any(|value| !is_nonempty(value)) || entry.variables.is_empty() {
        findings.push(AuditFinding::fail(
            "variables_nonempty",
            format!("{id}: variables must contain non-empty values"),
        ));
    }

    if entry.limitations.is_empty() || entry.limitations.iter().any(|value| !is_nonempty(value)) {
        findings.push(AuditFinding::fail(
            "limitations_nonempty",
            format!("{id}: limitations must contain non-empty values"),
        ));
    }

    if entry.admission_requirements.is_empty()
        || entry
            .admission_requirements
            .iter()
            .any(|value| !is_nonempty(value))
    {
        findings.push(AuditFinding::fail(
            "admission_requirements_nonempty",
            format!("{id}: admission_requirements must contain non-empty values"),
        ));
    }

    for (field, value) in [
        (
            "release_or_publication_date",
            entry.source_version.release_or_publication_date.as_deref(),
        ),
        (
            "metadata_updated",
            entry.source_version.metadata_updated.as_deref(),
        ),
    ] {
        if value.is_some_and(|date| !valid_date_precision(date, true)) {
            findings.push(AuditFinding::fail(
                "source_version_date",
                format!("{id}: {field} must be YYYY, YYYY-MM, YYYY-MM-DD or null"),
            ));
        }
    }

    if entry.raw_snapshot_ingested {
        findings.push(AuditFinding::fail(
            "discovery_only_snapshot_required",
            format!("{id}: v0.1 discovery records must keep raw_snapshot_ingested=false"),
        ));
    }
    if entry.snapshot_digest.is_some() {
        findings.push(AuditFinding::fail(
            "discovery_only_digest_must_be_null",
            format!("{id}: v0.1 discovery records must keep snapshot_digest=null"),
        ));
    }

    if entry.availability_state == AvailabilityState::TemporarilyUnavailable
        && (entry.source_type != SourceType::PortalStatus
            || entry.evidence_basis != EvidenceBasis::AvailabilityNotice)
    {
        findings.push(AuditFinding::fail(
            "unavailable_source_state_consistency",
            format!(
                "{id}: temporarily_unavailable requires source_type=portal_status and evidence_basis=availability_notice"
            ),
        ));
    }
    if entry.source_type == SourceType::PortalStatus
        && (entry.availability_state != AvailabilityState::TemporarilyUnavailable
            || entry.evidence_basis != EvidenceBasis::AvailabilityNotice)
    {
        findings.push(AuditFinding::fail(
            "portal_status_source_state_consistency",
            format!(
                "{id}: portal_status must be represented as temporarily_unavailable / availability_notice"
            ),
        ));
    }
    if entry.evidence_basis == EvidenceBasis::AvailabilityNotice
        && entry.source_type != SourceType::PortalStatus
    {
        findings.push(AuditFinding::fail(
            "availability_notice_source_type",
            format!("{id}: availability_notice requires source_type=portal_status"),
        ));
    }
    if entry.license_status.status == LicenseState::OpenAccessCatalogueOnly
        && entry.source_type != SourceType::DatasetCatalogue
    {
        findings.push(AuditFinding::fail(
            "catalogue_license_source_type",
            format!("{id}: open_access_catalogue_only requires dataset_catalogue"),
        ));
    }
}

/// Audit the exact schema and registry bytes. The receipt digest excludes no
/// fields except its own digest, so byte-identical inputs produce the same
/// receipt subject and digest on every run of this auditor version.
pub fn audit_bytes(schema_bytes: &[u8], registry_bytes: &[u8]) -> Result<AuditReceipt, AuditError> {
    let schema_value =
        parse_unique_json(schema_bytes).map_err(|cause| error(format!("schema: {cause}")))?;
    let registry_value =
        parse_unique_json(registry_bytes).map_err(|cause| error(format!("registry: {cause}")))?;

    let mut findings = Vec::new();
    check_schema_contract(&schema_value, &mut findings);

    let registry: Registry = serde_json::from_value(registry_value)
        .map_err(|cause| error(format!("registry typed decoding failed: {cause}")))?;
    validate_registry(&registry, &mut findings);

    let source_ids: Vec<String> = registry
        .entries
        .iter()
        .map(|entry| entry.source_id.clone())
        .collect();
    let disposition = if findings.is_empty() {
        "accepted"
    } else {
        "rejected"
    };
    let claim_ceiling = "Structural and semantic conformance to the pinned discovery-only source-registry v0.1 contract. This receipt does not authenticate source publishers, prove licences, validate resource science, establish site suitability, or authorize projects.";

    let schema_digest = sha256(schema_bytes);
    let registry_digest = sha256(registry_bytes);
    let subject = ReceiptSubject {
        receipt_schema: RECEIPT_SCHEMA,
        auditor_id: AUDITOR_ID,
        auditor_version: AUDITOR_VERSION,
        schema_sha256: &schema_digest,
        registry_sha256: &registry_digest,
        registry_status: &registry.registry_status,
        entry_count: registry.entries.len(),
        source_ids: &source_ids,
        disposition,
        findings: &findings,
        claim_ceiling,
    };
    let subject_bytes = serde_json::to_vec(&subject)
        .map_err(|cause| error(format!("receipt subject serialization failed: {cause}")))?;
    let receipt_sha256 = sha256(&subject_bytes);

    Ok(AuditReceipt {
        receipt_schema: RECEIPT_SCHEMA.to_owned(),
        auditor_id: AUDITOR_ID.to_owned(),
        auditor_version: AUDITOR_VERSION.to_owned(),
        schema_sha256: subject.schema_sha256.to_owned(),
        registry_sha256: subject.registry_sha256.to_owned(),
        registry_status: registry.registry_status,
        entry_count: registry.entries.len(),
        source_ids,
        disposition: disposition.to_owned(),
        findings,
        claim_ceiling: claim_ceiling.to_owned(),
        receipt_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA_BYTES: &[u8] =
        include_bytes!("../../../schemas/energy-resource-source-registry-v0.1.schema.json");
    const REGISTRY_BYTES: &[u8] = include_bytes!(
        "../../../docs/research/energy/SA_ENERGY_RESOURCE_SOURCE_REGISTER_V0_1.json"
    );

    fn parse_registry_value() -> Value {
        serde_json::from_slice(REGISTRY_BYTES).expect("checked-in registry JSON must parse")
    }

    fn audit_mutated_registry(value: Value) -> AuditReceipt {
        let bytes = serde_json::to_vec(&value).expect("mutated JSON must serialize");
        audit_bytes(SCHEMA_BYTES, &bytes).expect("parseable registry returns a receipt")
    }

    #[test]
    fn checked_in_registry_satisfies_v0_1_contract() {
        let receipt = audit_bytes(SCHEMA_BYTES, REGISTRY_BYTES)
            .expect("checked-in schema and source registry must decode");
        assert_eq!(receipt.disposition, "accepted");
        assert_eq!(receipt.entry_count, 22);
        assert_eq!(receipt.source_ids.len(), 22);
        assert_eq!(receipt.findings.len(), 0);
    }

    #[test]
    fn wasa3_resource_raster_is_distinct_from_unavailable_timeseries_portal() {
        let value = parse_registry_value();
        let entries = value["entries"].as_array().expect("entries array");
        let raster = entries
            .iter()
            .find(|entry| entry["source_id"] == "wasa3-high-resolution-wind-resource-map-2020")
            .expect("WASA3 raster source record");
        let portal = entries
            .iter()
            .find(|entry| entry["source_id"] == "wasa-south-africa-time-series-portal-availability")
            .expect("WASA time-series portal source record");

        assert_eq!(raster["source_type"], "dataset");
        assert_eq!(raster["evidence_basis"], "modeled_resource_atlas");
        assert_eq!(
            raster["availability_state"],
            "available_pending_snapshot_pin"
        );
        assert_eq!(portal["source_type"], "portal_status");
        assert_eq!(portal["evidence_basis"], "availability_notice");
        assert_eq!(portal["availability_state"], "temporarily_unavailable");

        let receipt = audit_bytes(SCHEMA_BYTES, REGISTRY_BYTES)
            .expect("updated registry should remain structurally valid");
        assert_eq!(receipt.disposition, "accepted");
        assert_eq!(receipt.entry_count, 22);
    }

    #[test]
    fn gwas_validation_cohort_does_not_claim_south_africa() {
        let value = parse_registry_value();
        let gwa = value["entries"]
            .as_array()
            .expect("entries array")
            .iter()
            .find(|entry| entry["source_id"] == "global-wind-atlas-4-0")
            .expect("GWA4 source record");
        let limitations = gwa["limitations"].as_array().expect("limitations array");
        assert!(limitations.iter().any(|item| {
            item.as_str()
                .is_some_and(|text| text.contains("does not include South Africa"))
        }));
    }

    #[test]
    fn south_african_hydropower_atlas_is_context_not_admitted_capacity() {
        let value = parse_registry_value();
        let entries = value["entries"].as_array().expect("entries array");
        let atlas = entries
            .iter()
            .find(|entry| entry["source_id"] == "south-african-hydropower-atlas-report-tt916-23")
            .expect("SA Hydropower Atlas report record");
        assert_eq!(atlas["evidence_basis"], "modeled_resource_atlas");
        assert_eq!(atlas["availability_state"], "context_only");
        assert_eq!(atlas["raw_snapshot_ingested"], false);
        assert!(
            atlas["claim_ceiling"]
                .as_str()
                .is_some_and(|claim| claim.contains("not measured site flow/head"))
        );
    }

    #[test]
    fn literature_review_is_not_mislabelled_as_a_field_measurement() {
        let value = parse_registry_value();
        let entries = value["entries"].as_array().expect("entries array");
        let review = entries
            .iter()
            .find(|entry| entry["source_id"] == "wdn-hydropower-review-2026")
            .expect("2026 WDN hydropower review record");
        assert_eq!(review["evidence_basis"], "literature_review");
        assert_ne!(review["evidence_basis"], "published_field_observation");
        let receipt =
            audit_bytes(SCHEMA_BYTES, REGISTRY_BYTES).expect("updated registry should conform");
        assert_eq!(receipt.disposition, "accepted");
        assert_eq!(receipt.entry_count, 22);
    }

    #[test]
    fn identical_inputs_produce_identical_receipt_digest() {
        let first = audit_bytes(SCHEMA_BYTES, REGISTRY_BYTES).expect("first audit");
        let second = audit_bytes(SCHEMA_BYTES, REGISTRY_BYTES).expect("second audit");
        assert_eq!(first.receipt_sha256, second.receipt_sha256);
        assert_eq!(first.schema_sha256, second.schema_sha256);
        assert_eq!(first.registry_sha256, second.registry_sha256);
    }

    #[test]
    fn duplicate_json_members_are_rejected() {
        let duplicate = br#"{"schema_version":"0.1.0","schema_version":"0.1.0"}"#;
        assert!(parse_unique_json(duplicate).is_err());
    }

    #[test]
    fn trailing_json_content_is_rejected() {
        let trailing = br#"{"ok":true} false"#;
        assert!(parse_unique_json(trailing).is_err());
    }

    #[test]
    fn input_size_limit_is_enforced() {
        let oversized = vec![b' '; MAX_INPUT_BYTES + 1];
        assert!(parse_unique_json(&oversized).is_err());
    }

    #[test]
    fn unknown_fields_fail_closed() {
        let mut value = parse_registry_value();
        value["unexpected"] = Value::Bool(true);
        let bytes = serde_json::to_vec(&value).expect("serialize registry");
        assert!(audit_bytes(SCHEMA_BYTES, &bytes).is_err());
    }

    #[test]
    fn required_nullable_fields_cannot_be_omitted() {
        let mut value = parse_registry_value();
        value["entries"][0]
            .as_object_mut()
            .expect("entry object")
            .remove("snapshot_digest");
        let bytes = serde_json::to_vec(&value).expect("serialize registry");
        assert!(audit_bytes(SCHEMA_BYTES, &bytes).is_err());
    }

    #[test]
    fn duplicate_source_ids_are_rejected() {
        let mut value = parse_registry_value();
        let first_id = value["entries"][0]["source_id"]
            .as_str()
            .expect("source ID")
            .to_owned();
        value["entries"][1]["source_id"] = Value::String(first_id);
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "source_id_unique")
        );
    }

    #[test]
    fn discovery_record_cannot_claim_ingestion_or_digest() {
        let mut value = parse_registry_value();
        value["entries"][0]["raw_snapshot_ingested"] = Value::Bool(true);
        value["entries"][0]["snapshot_digest"] =
            Value::String(format!("sha256:{}", "a".repeat(64)));
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "discovery_only_snapshot_required")
        );
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "discovery_only_digest_must_be_null")
        );
    }

    #[test]
    fn unavailable_portal_cannot_be_relabelled_as_a_dataset() {
        let mut value = parse_registry_value();
        let entry = value["entries"]
            .as_array_mut()
            .expect("entries array")
            .iter_mut()
            .find(|entry| entry["source_id"] == "wasa-south-africa-time-series-portal-availability")
            .expect("WASA portal status entry");
        entry["source_type"] = Value::String("dataset".to_owned());
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "unavailable_source_state_consistency")
        );
    }

    #[test]
    fn http_locator_is_rejected() {
        let mut value = parse_registry_value();
        value["entries"][0]["source_url"] = Value::String("http://example.org/data".to_owned());
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "source_url_https")
        );
    }

    #[test]
    fn malformed_source_date_is_rejected() {
        let mut value = parse_registry_value();
        value["entries"][0]["source_version"]["metadata_updated"] =
            Value::String("2026-13-45".to_owned());
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "source_version_date")
        );
    }

    #[test]
    fn schema_enum_drift_is_rejected() {
        let mut schema: Value =
            serde_json::from_slice(SCHEMA_BYTES).expect("schema JSON must parse");
        schema["$defs"]["source_entry"]["properties"]["source_type"]["enum"]
            .as_array_mut()
            .expect("source type enum")
            .push(Value::String("invented_type".to_owned()));
        let bytes = serde_json::to_vec(&schema).expect("serialize schema");
        let receipt = audit_bytes(&bytes, REGISTRY_BYTES).expect("schema JSON is parseable");
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "schema_enum_values")
        );
    }

    #[test]
    fn non_discovery_registry_status_is_rejected() {
        let mut value = parse_registry_value();
        value["registry_status"] = Value::String("admitted".to_owned());
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "registry_discovery_only_status")
        );
    }

    #[test]
    fn malformed_top_level_date_is_rejected() {
        let mut value = parse_registry_value();
        value["as_of"] = Value::String("2026-02-30".to_owned());
        let receipt = audit_mutated_registry(value);
        assert_eq!(receipt.disposition, "rejected");
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == "registry_as_of_date")
        );
    }
}
