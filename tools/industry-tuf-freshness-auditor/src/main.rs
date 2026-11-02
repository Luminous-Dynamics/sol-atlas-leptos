use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process::ExitCode;

const VERIFIER_SCHEMA: &str = "luminous.ieac-tuf-verifier-report.v0.1";
const POLICY_SCHEMA: &str = "luminous.ieac-tuf-freshness-policy.v0.1";
const STATE_SCHEMA: &str = "luminous.ieac-tuf-high-water-state.v0.1";
const REPORT_SCHEMA: &str = "luminous.ieac-tuf-freshness-report.v0.1";

#[derive(Debug)]
enum StrictJson {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<StrictJson>),
    Object(BTreeMap<String, StrictJson>),
}
impl StrictJson {
    fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(v) => Value::Bool(v),
            Self::Number(v) => Value::Number(v),
            Self::String(v) => Value::String(v),
            Self::Array(v) => Value::Array(v.into_iter().map(Self::into_value).collect()),
            Self::Object(v) => {
                let mut out = Map::new();
                for (k, value) in v { out.insert(k, value.into_value()); }
                Value::Object(out)
            }
        }
    }
}
impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("JSON value with no duplicate object members")
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::Null)
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::Null)
            }
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::Number(Number::from(v)))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::Number(Number::from(v)))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: de::Error {
                Number::from_f64(v).map(StrictJson::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::String(v.to_string()))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: de::Error {
                Ok(StrictJson::String(v))
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<StrictJson>()? {
                    values.push(value);
                }
                Ok(StrictJson::Array(values))
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de> {
                let mut values = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate JSON member: {key}")));
                    }
                    values.insert(key, map.next_value::<StrictJson>()?);
                }
                Ok(StrictJson::Object(values))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}
fn parse_strict_json(raw: &[u8], label: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(raw);
    let parsed = StrictJson::deserialize(&mut deserializer)
        .map_err(|e| format!("invalid strict JSON in {label}: {e}"))?;
    deserializer.end()
        .map_err(|e| format!("trailing data in {label}: {e}"))?;
    Ok(parsed.into_value())
}

fn obj<'a>(v: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object()
        .ok_or_else(|| format!("{path} must be an object"))
}
fn field<'a>(v: &'a Value, key: &str, path: &str) -> Result<&'a Value, String> {
    obj(v, path)?
        .get(key)
        .ok_or_else(|| format!("{path}.{key} is required"))
}
fn string<'a>(v: &'a Value, key: &str, path: &str) -> Result<&'a str, String> {
    field(v, key, path)?
        .as_str()
        .ok_or_else(|| format!("{path}.{key} must be a string"))
}
fn uint(v: &Value, key: &str, path: &str) -> Result<u64, String> {
    field(v, key, path)?
        .as_u64()
        .ok_or_else(|| format!("{path}.{key} must be a non-negative integer"))
}
fn bool_field(v: &Value, key: &str, path: &str) -> Result<bool, String> {
    field(v, key, path)?
        .as_bool()
        .ok_or_else(|| format!("{path}.{key} must be a boolean"))
}
fn array<'a>(v: &'a Value, key: &str, path: &str) -> Result<&'a Vec<Value>, String> {
    field(v, key, path)?
        .as_array()
        .ok_or_else(|| format!("{path}.{key} must be an array"))
}
fn is_sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn require_digest(v: &Value, key: &str, path: &str) -> Result<&str, String> {
    let s = string(v, key, path)?;
    if !is_sha256(s) {
        return Err(format!("{path}.{key} is not a lowercase SHA-256 digest"));
    }
    Ok(s)
}
fn deny_unknown(v: &Value, allowed: &[&str], path: &str) -> Result<(), String> {
    let map = obj(v, path)?;
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{path} contains unknown field {key}"));
        }
    }
    Ok(())
}

fn leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
fn month_days(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}
// A deliberately narrow parser: whole-second normalized UTC only. TUF metadata
// remains verified by the external client; this parser only compares expiry.
fn parse_utc(s: &str) -> Result<i64, String> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-' || b[7] != b'-' || b[10] != b'T'
        || b[13] != b':' || b[16] != b':' || b[19] != b'Z'
    {
        return Err("timestamp must be YYYY-MM-DDTHH:MM:SSZ".to_string());
    }
    let part = |a: usize, z: usize| -> Result<i64, String> {
        let bytes = &b[a..z];
        if !bytes.iter().all(|x| x.is_ascii_digit()) {
            return Err("timestamp components must be ASCII digits".to_string());
        }
        let s = std::str::from_utf8(bytes).map_err(|_| "timestamp is not ASCII")?;
        s.parse::<i64>().map_err(|_| "timestamp component out of range".to_string())
    };
    let y = part(0, 4)?;
    let m = part(5, 7)?;
    let d = part(8, 10)?;
    let h = part(11, 13)?;
    let min = part(14, 16)?;
    let sec = part(17, 19)?;
    if y < 1970 || !(1..=12).contains(&m) || d < 1 || d > month_days(y, m)
        || h > 23 || min > 59 || sec > 59
    {
        return Err("timestamp is not a supported Gregorian UTC instant".to_string());
    }
    let y_adj = y - if m <= 2 { 1 } else { 0 };
    let era = y_adj.div_euclid(400);
    let yoe = y_adj - era * 400;
    let m_adj = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * m_adj + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Ok(days * 86_400 + h * 3_600 + min * 60 + sec)
}

#[derive(Debug)]
struct Interval {
    lo: i64,
    hi: i64,
}
fn interval(v: &Value, path: &str) -> Result<Interval, String> {
    deny_unknown(
        v,
        &["time_utc", "uncertainty_before_seconds", "uncertainty_after_seconds",
          "clock_id", "verification_status", "proof_sha256",
          "verifier_build_sha256", "verification_policy_sha256",
          "trust_bundle_sha256"],
        path,
    )?;
    let center = parse_utc(string(v, "time_utc", path)?)?;
    let before = uint(v, "uncertainty_before_seconds", path)?;
    let after = uint(v, "uncertainty_after_seconds", path)?;
    if before > 86_400 || after > 86_400 {
        return Err(format!("{path} uncertainty exceeds one day"));
    }
    if string(v, "clock_id", path)?.is_empty() {
        return Err(format!("{path}.clock_id is empty"));
    }
    if string(v, "verification_status", path)? != "verified" {
        return Err(format!("{path} clock evidence is not verified"));
    }
    for key in ["proof_sha256", "verifier_build_sha256",
                "verification_policy_sha256", "trust_bundle_sha256"] {
        require_digest(v, key, path)?;
    }
    Ok(Interval {
        lo: center - before as i64,
        hi: center + after as i64,
    })
}

#[derive(Debug)]
struct TargetRoleInfo {
    version: u64,
    metadata_sha256: String,
    expires_at: String,
    target_path: String,
    target_length: u64,
    target_sha256: String,
}

#[derive(Debug)]
struct AuditResult {
    status: &'static str,
    structural_status: &'static str,
    reason_codes: Vec<String>,
    root_version: u64,
    timestamp_version: u64,
    snapshot_version: u64,
    target_role_versions: BTreeMap<String, u64>,
    manifest_sha256: String,
    checkpoint_sha256: String,
    completeness_attestation_sha256: String,
    availability_report_sha256: String,
    checkpoint_sequence: u64,
    checkpoint_epoch: String,
    verifier_report_sha256: String,
    policy_sha256: String,
    trusted_state_sha256: String,
    evaluation_interval: Interval,
}

fn check_equal(v: &Value, key: &str, expected: &str, path: &str) -> Result<(), String> {
    let actual = string(v, key, path)?;
    if actual != expected {
        return Err(format!("{path}.{key} mismatch"));
    }
    Ok(())
}

fn check_metadata_version(
    name: &str,
    new_version: u64,
    new_sha: &str,
    old_version: u64,
    old_sha: &str,
) -> Result<(), String> {
    if new_version < old_version {
        return Err(format!("{name} metadata rollback: {new_version} < {old_version}"));
    }
    if new_version == old_version && new_sha != old_sha {
        return Err(format!("{name} same-version metadata equivocation"));
    }
    Ok(())
}

fn expiry_result(expiry: &str, eval: &Interval, max_horizon: Option<u64>) -> Result<Option<String>, String> {
    let expiry = parse_utc(expiry)?;
    if eval.lo >= expiry {
        return Ok(Some("metadata_expired".to_string()));
    }
    if eval.hi >= expiry {
        return Ok(Some("expiry_boundary_indeterminate".to_string()));
    }
    if let Some(max) = max_horizon {
        if expiry - eval.lo > max as i64 {
            return Ok(Some("metadata_expiry_horizon_exceeds_policy".to_string()));
        }
    }
    Ok(None)
}

fn audit(
    report: &Value,
    policy: &Value,
    state: &Value,
    report_raw: &[u8],
    policy_raw: &[u8],
    state_raw: &[u8],
) -> Result<AuditResult, String> {
    deny_unknown(report,
        &["schema_version","repository_id","root","timestamp","snapshot",
          "target_roles","channel_binding","root_update_chain",
          "verification_checks","evaluation_time","verifier"], "verifier_report")?;
    deny_unknown(policy,
        &["schema_version","repository_id","relying_party_id","channel_id",
          "tenant_scope","purpose","ledger_id","sequence_namespace",
          "manifest_target_path","manifest_target_role",
          "checkpoint_target_path","checkpoint_target_role",
          "completeness_attestation_target_path","completeness_attestation_target_role",
          "availability_report_target_path","availability_report_target_role",
          "max_timestamp_expiry_horizon_seconds",
          "time_verifier_build_sha256","time_verification_policy_sha256",
          "time_verification_trust_bundle_sha256",
          "tuf_verifier_build_sha256","tuf_verification_policy_sha256",
          "tuf_trust_bundle_sha256","claim_ceiling","action_authority"],
        "policy")?;
    deny_unknown(state,
        &["schema_version","state_id","state_revision","repository_id",
          "relying_party_id","channel_id","tenant_scope","purpose","ledger_id",
          "sequence_namespace","sequence_epoch","last_sequence",
          "root","timestamp","snapshot","target_roles",
          "manifest_sha256","checkpoint_state_sha256",
          "completeness_attestation_sha256","availability_report_sha256"],
        "trusted_state")?;

    if string(report,"schema_version","verifier_report")? != VERIFIER_SCHEMA
        || string(policy,"schema_version","policy")? != POLICY_SCHEMA
        || string(state,"schema_version","trusted_state")? != STATE_SCHEMA
    {
        return Err("unsupported input schema version".to_string());
    }
    if string(policy,"claim_ceiling","policy")? != "tuf_freshness_and_anti_rollback_only"
        || string(policy,"action_authority","policy")? != "none"
    {
        return Err("policy claim ceiling or action-authority boundary is invalid".to_string());
    }
    if uint(state,"state_revision","trusted_state")? == 0
        || string(state,"state_id","trusted_state")?.is_empty()
    {
        return Err("trusted high-water state identity/revision is invalid".to_string());
    }
    for (field, expected) in [
        ("repository_id", string(policy,"repository_id","policy")?),
    ] {
        check_equal(report,field,expected,"verifier_report")?;
        check_equal(state,field,expected,"trusted_state")?;
    }
    for (field, expected) in [
        ("relying_party_id", string(policy,"relying_party_id","policy")?),
        ("channel_id", string(policy,"channel_id","policy")?),
        ("tenant_scope", string(policy,"tenant_scope","policy")?),
        ("purpose", string(policy,"purpose","policy")?),
        ("ledger_id", string(policy,"ledger_id","policy")?),
        ("sequence_namespace", string(policy,"sequence_namespace","policy")?),
    ] {
        check_equal(state,field,expected,"trusted_state")?;
    }

    let policy_tuf_build = require_digest(policy,"tuf_verifier_build_sha256","policy")?;
    let policy_tuf_policy = require_digest(policy,"tuf_verification_policy_sha256","policy")?;
    let policy_tuf_bundle = require_digest(policy,"tuf_trust_bundle_sha256","policy")?;
    let verifier = field(report,"verifier","verifier_report")?;
    deny_unknown(verifier,&["status","build_sha256","policy_sha256","trust_bundle_sha256"],"verifier")?;
    let mut unknown = Vec::new();
    if string(verifier,"status","verifier")? != "verified" {
        unknown.push("tuf_verifier_not_verified".to_string());
    }
    if string(verifier,"build_sha256","verifier")? != policy_tuf_build
        || string(verifier,"policy_sha256","verifier")? != policy_tuf_policy
        || string(verifier,"trust_bundle_sha256","verifier")? != policy_tuf_bundle
    {
        return Err("TUF verifier build/policy/trust bundle does not match policy pins".to_string());
    }

    let eval = interval(field(report,"evaluation_time","verifier_report")?,"evaluation_time")?;
    // Time facts must have been checked by the separately pinned time verifier.
    let time = field(report,"evaluation_time","verifier_report")?;
    for (key, expected) in [
        ("verifier_build_sha256", string(policy,"time_verifier_build_sha256","policy")?),
        ("verification_policy_sha256", string(policy,"time_verification_policy_sha256","policy")?),
        ("trust_bundle_sha256", string(policy,"time_verification_trust_bundle_sha256","policy")?),
    ] {
        if string(time,key,"evaluation_time")? != expected {
            return Err(format!("evaluation_time.{key} does not match policy pin"));
        }
    }

    let root = field(report,"root","verifier_report")?;
    let timestamp = field(report,"timestamp","verifier_report")?;
    let snapshot_meta = field(report,"snapshot","verifier_report")?;
    deny_unknown(root,&["version","sha256","expires_at","timestamp_keyset_sha256","snapshot_keyset_sha256"],"root")?;
    require_digest(root,"sha256","root")?;
    let new_timestamp_keyset=require_digest(root,"timestamp_keyset_sha256","root")?;
    let new_snapshot_keyset=require_digest(root,"snapshot_keyset_sha256","root")?;
    for (v,path) in [(timestamp,"timestamp"),(snapshot_meta,"snapshot")] {
        deny_unknown(v,&["version","sha256","expires_at"],path)?;
        require_digest(v,"sha256",path)?;
    }
    let root_version = uint(root,"version","root")?;
    let timestamp_version = uint(timestamp,"version","timestamp")?;
    let snapshot_version = uint(snapshot_meta,"version","snapshot")?;
    let old_root = field(state,"root","trusted_state")?;
    let old_timestamp = field(state,"timestamp","trusted_state")?;
    let old_snapshot = field(state,"snapshot","trusted_state")?;
    deny_unknown(old_root,&["version","sha256","expires_at","timestamp_keyset_sha256","snapshot_keyset_sha256"],"state.root")?;
    require_digest(old_root,"sha256","state.root")?;
    let old_timestamp_keyset=require_digest(old_root,"timestamp_keyset_sha256","state.root")?;
    let old_snapshot_keyset=require_digest(old_root,"snapshot_keyset_sha256","state.root")?;
    for (v,path) in [(old_timestamp,"state.timestamp"),(old_snapshot,"state.snapshot")] {
        deny_unknown(v,&["version","sha256","expires_at"],path)?;
        require_digest(v,"sha256",path)?;
    }
    check_metadata_version("root",root_version,string(root,"sha256","root")?,
        uint(old_root,"version","state.root")?,string(old_root,"sha256","state.root")?)?;
    let timestamp_or_snapshot_keys_rotated =
        new_timestamp_keyset != old_timestamp_keyset || new_snapshot_keyset != old_snapshot_keyset;
    if timestamp_or_snapshot_keys_rotated && root_version <= uint(old_root,"version","state.root")? {
        return Err("TUF timestamp/snapshot key rotation requires a newer trusted root".to_string());
    }
    if timestamp_or_snapshot_keys_rotated
        && string(root,"sha256","root")? == string(old_root,"sha256","state.root")?
    {
        return Err("TUF role-key set changed without a root metadata digest change".to_string());
    }
    // TUF recovery semantics: if timestamp and/or snapshot keys rotate under a
    // newly trusted root, clients discard both cached timestamp and snapshot
    // metadata. Their prior version floors are reset only for this verified case.
    if !timestamp_or_snapshot_keys_rotated {
        check_metadata_version("timestamp",timestamp_version,string(timestamp,"sha256","timestamp")?,
            uint(old_timestamp,"version","state.timestamp")?,string(old_timestamp,"sha256","state.timestamp")?)?;
        check_metadata_version("snapshot",snapshot_version,string(snapshot_meta,"sha256","snapshot")?,
            uint(old_snapshot,"version","state.snapshot")?,string(old_snapshot,"sha256","state.snapshot")?)?;
    }

    // Root rotation must be a complete, contiguous N -> N+1 chain with both
    // predecessor and successor thresholds verified by the external TUF client.
    let old_root_version = uint(old_root,"version","state.root")?;
    let old_root_sha = string(old_root,"sha256","state.root")?;
    if root_version > old_root_version {
        let updates = array(report,"root_update_chain","verifier_report")?;
        if updates.len() as u64 != root_version - old_root_version {
            return Err("root update chain is incomplete".to_string());
        }
        let mut expected_version = old_root_version;
        let mut expected_sha = old_root_sha.to_string();
        for (i,u) in updates.iter().enumerate() {
            let path = format!("root_update_chain[{i}]");
            deny_unknown(u,&["from_version","from_sha256","to_version","to_sha256",
                "old_threshold_verified","new_threshold_verified"],&path)?;
            if uint(u,"from_version",&path)? != expected_version
                || string(u,"from_sha256",&path)? != expected_sha
                || uint(u,"to_version",&path)? != expected_version + 1
                || !bool_field(u,"old_threshold_verified",&path)?
                || !bool_field(u,"new_threshold_verified",&path)?
            {
                return Err("root update chain is not contiguous and threshold-verified".to_string());
            }
            expected_version += 1;
            expected_sha = require_digest(u,"to_sha256",&path)?.to_string();
        }
        if expected_version != root_version || expected_sha != string(root,"sha256","root")? {
            return Err("root update chain does not end at the reported trusted root".to_string());
        }
    } else if !array(report,"root_update_chain","verifier_report")?.is_empty() {
        return Err("root update chain must be empty when root version is unchanged".to_string());
    }

    let max_horizon = uint(policy,"max_timestamp_expiry_horizon_seconds","policy")?;
    for (name, metadata, max) in [
        ("root",root,None),
        ("timestamp",timestamp,Some(max_horizon)),
        ("snapshot",snapshot_meta,None),
    ] {
        if let Some(reason)=expiry_result(string(metadata,"expires_at",name)?,&eval,max)? {
            unknown.push(format!("{name}_{reason}"));
        }
    }

    let checks=field(report,"verification_checks","verifier_report")?;
    deny_unknown(checks,&["timestamp_snapshot_binding_verified",
        "snapshot_targets_binding_verified","delegation_authorization_verified",
        "target_hash_length_verified","root_chain_verified",
        "timestamp_snapshot_key_rotation_recovery_verified"],"verification_checks")?;
    if timestamp_or_snapshot_keys_rotated
        && !bool_field(checks,"timestamp_snapshot_key_rotation_recovery_verified","verification_checks")?
    {
        unknown.push("tuf_timestamp_snapshot_key_rotation_recovery_not_verified".to_string());
    }
    for k in ["timestamp_snapshot_binding_verified","snapshot_targets_binding_verified",
        "delegation_authorization_verified","target_hash_length_verified","root_chain_verified"] {
        if !bool_field(checks,k,"verification_checks")? {
            unknown.push(format!("tuf_check_{k}_not_verified"));
        }
    }

    let roles=array(report,"target_roles","verifier_report")?;
    let mut new_role_map=BTreeMap::<String,TargetRoleInfo>::new();
    for (i,r) in roles.iter().enumerate() {
        let p=format!("target_roles[{i}]");
        deny_unknown(r,&["role_name","version","metadata_sha256","expires_at",
            "target_path","target_length","target_sha256"],&p)?;
        let name=string(r,"role_name",&p)?.to_string();
        let meta_sha=require_digest(r,"metadata_sha256",&p)?.to_string();
        let info=TargetRoleInfo {
            version:uint(r,"version",&p)?,
            metadata_sha256:meta_sha,
            expires_at:string(r,"expires_at",&p)?.to_string(),
            target_path:string(r,"target_path",&p)?.to_string(),
            target_length:uint(r,"target_length",&p)?,
            target_sha256:require_digest(r,"target_sha256",&p)?.to_string(),
        };
        if new_role_map.insert(name,info).is_some() {
            return Err("duplicate TUF target role".to_string());
        }
        if let Some(reason)=expiry_result(string(r,"expires_at",&p)?,&eval,None)? {
            unknown.push(format!("target_role_{i}_{reason}"));
        }
    }
    let old_roles=array(state,"target_roles","trusted_state")?;
    let mut old_role_map=BTreeMap::<String,(u64,String)>::new();
    for (i,r) in old_roles.iter().enumerate() {
        let p=format!("state.target_roles[{i}]");
        deny_unknown(r,&["role_name","version","metadata_sha256"],&p)?;
        let name=string(r,"role_name",&p)?.to_string();
        let entry=(uint(r,"version",&p)?,require_digest(r,"metadata_sha256",&p)?.to_string());
        if old_role_map.insert(name,entry).is_some() {
            return Err("duplicate persisted TUF target role".to_string());
        }
    }
    for (role,(old_v,old_sha)) in &old_role_map {
        let new_info=new_role_map.get(role)
            .ok_or_else(||format!("previously trusted delegated target role {role} disappeared"))?;
        check_metadata_version(role,new_info.version,&new_info.metadata_sha256,*old_v,old_sha)?;
    }

    let manifest_path=string(policy,"manifest_target_path","policy")?;
    let manifest_role=string(policy,"manifest_target_role","policy")?;
    let checkpoint_path=string(policy,"checkpoint_target_path","policy")?;
    let checkpoint_role=string(policy,"checkpoint_target_role","policy")?;
    let completeness_path=string(policy,"completeness_attestation_target_path","policy")?;
    let completeness_role=string(policy,"completeness_attestation_target_role","policy")?;
    let availability_report_path=string(policy,"availability_report_target_path","policy")?;
    let availability_report_role=string(policy,"availability_report_target_role","policy")?;
    let unique_roles:BTreeSet<&str>=[manifest_role,checkpoint_role,completeness_role,availability_report_role].into_iter().collect();
    let unique_paths:BTreeSet<&str>=[manifest_path,checkpoint_path,completeness_path,availability_report_path].into_iter().collect();
    if unique_roles.len()!=4 || unique_paths.len()!=4 {
        return Err("manifest, checkpoint, completeness attestation and audit report must use distinct TUF target roles and paths".to_string());
    }
    let binding=field(report,"channel_binding","verifier_report")?;
    deny_unknown(binding,&["relying_party_id","channel_id","tenant_scope","purpose",
        "ledger_id","sequence_namespace","sequence_epoch","last_sequence",
        "channel_manifest","checkpoint_state","completeness_attestation",
        "availability_audit_report","epoch_transition"],"channel_binding")?;
    for (k,expected) in [
        ("relying_party_id",string(policy,"relying_party_id","policy")?),
        ("channel_id",string(policy,"channel_id","policy")?),
        ("tenant_scope",string(policy,"tenant_scope","policy")?),
        ("purpose",string(policy,"purpose","policy")?),
        ("ledger_id",string(policy,"ledger_id","policy")?),
        ("sequence_namespace",string(policy,"sequence_namespace","policy")?),
    ] {
        check_equal(binding,k,expected,"channel_binding")?;
    }
    let new_epoch=string(binding,"sequence_epoch","channel_binding")?;
    let new_seq=uint(binding,"last_sequence","channel_binding")?;

    let manifest=field(binding,"channel_manifest","channel_binding")?;
    let checkpoint=field(binding,"checkpoint_state","channel_binding")?;
    let completeness=field(binding,"completeness_attestation","channel_binding")?;
    let availability_report=field(binding,"availability_audit_report","channel_binding")?;
    for (t,p) in [
        (manifest,"channel_manifest"),(checkpoint,"checkpoint_state"),
        (completeness,"completeness_attestation"),(availability_report,"availability_audit_report")
    ] {
        deny_unknown(t,&["role_name","target_path","target_length","target_sha256"],p)?;
        require_digest(t,"target_sha256",p)?;
        if uint(t,"target_length",p)? == 0 {
            return Err(format!("{p}.target_length must be non-zero"));
        }
    }
    if string(manifest,"role_name","channel_manifest")? != manifest_role
        || string(manifest,"target_path","channel_manifest")? != manifest_path
        || string(checkpoint,"role_name","checkpoint_state")? != checkpoint_role
        || string(checkpoint,"target_path","checkpoint_state")? != checkpoint_path
        || string(completeness,"role_name","completeness_attestation")? != completeness_role
        || string(completeness,"target_path","completeness_attestation")? != completeness_path
        || string(availability_report,"role_name","availability_audit_report")? != availability_report_role
        || string(availability_report,"target_path","availability_audit_report")? != availability_report_path
    {
        return Err("TUF target role/path does not match the pinned channel policy".to_string());
    }
    let manifest_role_info=new_role_map.get(manifest_role)
        .ok_or_else(||"required channel-manifest TUF target role missing".to_string())?;
    let checkpoint_role_info=new_role_map.get(checkpoint_role)
        .ok_or_else(||"required checkpoint-state TUF target role missing".to_string())?;
    let completeness_role_info=new_role_map.get(completeness_role)
        .ok_or_else(||"required completeness-attestation TUF target role missing".to_string())?;
    let availability_report_role_info=new_role_map.get(availability_report_role)
        .ok_or_else(||"required availability-report TUF target role missing".to_string())?;
    let old_manifest_sha=string(state,"manifest_sha256","trusted_state")?;
    let old_checkpoint_sha=string(state,"checkpoint_state_sha256","trusted_state")?;
    let old_completeness_sha=string(state,"completeness_attestation_sha256","trusted_state")?;
    let old_availability_report_sha=string(state,"availability_report_sha256","trusted_state")?;
    if !is_sha256(old_manifest_sha)||!is_sha256(old_checkpoint_sha)
        || !is_sha256(old_completeness_sha)||!is_sha256(old_availability_report_sha) {
        return Err("persisted target digest is invalid".to_string());
    }
    let manifest_sha=string(manifest,"target_sha256","channel_manifest")?.to_string();
    let checkpoint_sha=string(checkpoint,"target_sha256","checkpoint_state")?.to_string();
    let completeness_sha=string(completeness,"target_sha256","completeness_attestation")?.to_string();
    let availability_report_sha=string(availability_report,"target_sha256","availability_audit_report")?.to_string();
    if manifest_role_info.target_path != manifest_path
        || manifest_role_info.target_path != string(manifest,"target_path","channel_manifest")?
        || manifest_role_info.target_length != uint(manifest,"target_length","channel_manifest")?
        || manifest_role_info.target_sha256 != manifest_sha
        || checkpoint_role_info.target_path != checkpoint_path
        || checkpoint_role_info.target_path != string(checkpoint,"target_path","checkpoint_state")?
        || checkpoint_role_info.target_length != uint(checkpoint,"target_length","checkpoint_state")?
        || checkpoint_role_info.target_sha256 != checkpoint_sha
        || completeness_role_info.target_path != completeness_path
        || completeness_role_info.target_path != string(completeness,"target_path","completeness_attestation")?
        || completeness_role_info.target_length != uint(completeness,"target_length","completeness_attestation")?
        || completeness_role_info.target_sha256 != completeness_sha
        || availability_report_role_info.target_path != availability_report_path
        || availability_report_role_info.target_path != string(availability_report,"target_path","availability_audit_report")?
        || availability_report_role_info.target_length != uint(availability_report,"target_length","availability_audit_report")?
        || availability_report_role_info.target_sha256 != availability_report_sha
    {
        return Err("TUF metadata target path/length/digest does not match channel binding".to_string());
    }
    if let Some((old_version,_))=old_role_map.get(manifest_role) {
        if manifest_role_info.version == *old_version && manifest_sha != old_manifest_sha {
            return Err("channel manifest target changed without a delegated metadata version advance".to_string());
        }
    }
    if let Some((old_version,_))=old_role_map.get(checkpoint_role) {
        if checkpoint_role_info.version == *old_version && checkpoint_sha != old_checkpoint_sha {
            return Err("checkpoint state target changed without a delegated metadata version advance".to_string());
        }
    }
    if let Some((old_version,_))=old_role_map.get(completeness_role) {
        if completeness_role_info.version == *old_version && completeness_sha != old_completeness_sha {
            return Err("completeness attestation changed without a delegated metadata version advance".to_string());
        }
    }
    if let Some((old_version,_))=old_role_map.get(availability_report_role) {
        if availability_report_role_info.version == *old_version && availability_report_sha != old_availability_report_sha {
            return Err("availability audit report changed without a delegated metadata version advance".to_string());
        }
    }
    let old_epoch=string(state,"sequence_epoch","trusted_state")?;
    let old_seq=uint(state,"last_sequence","trusted_state")?;
    let mut epoch_transition_valid=false;
    if new_epoch == old_epoch {
        if new_seq < old_seq {
            return Err("channel sequence rollback within the same epoch".to_string());
        }
        if new_seq == old_seq && checkpoint_sha != old_checkpoint_sha {
            return Err("same-sequence checkpoint equivocation".to_string());
        }
    } else {
        let transition=field(binding,"epoch_transition","channel_binding")?;
        deny_unknown(transition,&["status","from_epoch","to_epoch",
            "authorization_target_sha256"],"epoch_transition")?;
        epoch_transition_valid =
            string(transition,"status","epoch_transition")? == "verified"
            && string(transition,"from_epoch","epoch_transition")? == old_epoch
            && string(transition,"to_epoch","epoch_transition")? == new_epoch
            && string(transition,"authorization_target_sha256","epoch_transition")? == manifest_sha;
        if !epoch_transition_valid {
            return Err("channel epoch reset lacks a verified TUF-authorized transition".to_string());
        }
    }
    if string(state,"repository_id","trusted_state")? != string(report,"repository_id","verifier_report")? {
        return Err("trusted state repository identity mismatch".to_string());
    }

    let role_versions=new_role_map.iter().map(|(k,v)|(k.clone(),v.version)).collect();
    // The supplied verifier report is unauthenticated JSON. This adapter can
    // only assess structural consistency; it must never claim TUF "current"
    // or emit a high-water transition until in-process TUF verification lands.
    let structural_status=if unknown.is_empty() {"consistent"} else {"unknown"};
    let status="unverified";
    Ok(AuditResult{
        status,
        structural_status,
        reason_codes:unknown,
        root_version,
        timestamp_version,
        snapshot_version,
        target_role_versions:role_versions,
        manifest_sha256:manifest_sha,
        checkpoint_sha256:checkpoint_sha,
        completeness_attestation_sha256:completeness_sha,
        availability_report_sha256:availability_report_sha,
        checkpoint_sequence:new_seq,
        checkpoint_epoch:new_epoch.to_string(),
        verifier_report_sha256:digest(report_raw),
        policy_sha256:digest(policy_raw),
        trusted_state_sha256:digest(state_raw),
        evaluation_interval:eval,
    })
}

fn to_report(result: &AuditResult, state_id: &str) -> Value {
    let roles:Vec<Value>=result.target_role_versions.iter()
        .map(|(role,version)|json!({"role_name":role,"version":version})).collect();
    json!({
        "schema_version": REPORT_SCHEMA,
        "status": result.status,
        "structural_status": result.structural_status,
        "verification_boundary": "external_tuf_report_not_authenticated_or_cryptographically_verified_here",
        "reason_codes": result.reason_codes,
        "trusted_state_id": state_id,
        "verifier_report_sha256": result.verifier_report_sha256,
        "policy_sha256": result.policy_sha256,
        "trusted_state_sha256": result.trusted_state_sha256,
        "evaluation_interval": {
            "earliest_epoch_seconds": result.evaluation_interval.lo,
            "latest_epoch_seconds": result.evaluation_interval.hi
        },
        "reported_metadata": {
            "root_version": result.root_version,
            "timestamp_version": result.timestamp_version,
            "snapshot_version": result.snapshot_version,
            "target_roles": roles
        },
        "reported_channel_state": {
            "manifest_sha256": result.manifest_sha256,
            "checkpoint_state_sha256": result.checkpoint_sha256,
            "completeness_attestation_sha256": result.completeness_attestation_sha256,
            "availability_report_sha256": result.availability_report_sha256,
            "ledger_epoch": result.checkpoint_epoch,
            "last_sequence": result.checkpoint_sequence
        },
        "state_transition": Value::Null,
        "claim_ceiling": "tuf_freshness_and_anti_rollback_only",
        "action_authority": "none"
    })
}

fn read_json(path:&str)->Result<(Vec<u8>,Value),String>{
    let raw=fs::read(path).map_err(|e|format!("cannot read {path}: {e}"))?;
    let value=parse_strict_json(&raw,path)?;
    Ok((raw,value))
}
fn run()->Result<u8,String>{
    let args:Vec<String>=env::args().collect();
    if args.len()!=5 {
        return Err(format!("usage: {} <tuf-verifier-report.json> <policy-pin.json> <trusted-high-water-state.json> <output-report.json>",
            args.first().map(String::as_str).unwrap_or("industry-tuf-freshness-audit")));
    }
    let (report_raw,report)=read_json(&args[1])?;
    let (policy_raw,policy)=read_json(&args[2])?;
    let (state_raw,state)=read_json(&args[3])?;
    let state_id=string(&state,"state_id","trusted_state")?.to_string();
    let result=audit(&report,&policy,&state,&report_raw,&policy_raw,&state_raw)?;
    let structural_status=result.structural_status;
    let output=serde_json::to_vec_pretty(&to_report(&result,&state_id))
        .map_err(|e|format!("cannot serialize report: {e}"))?;
    fs::write(&args[4],output).map_err(|e|format!("cannot write report {}: {e}",args[4]))?;
    Ok(if structural_status=="consistent" {0} else {3})
}
fn main()->ExitCode{
    match run(){
        Ok(code)=>ExitCode::from(code),
        Err(e)=>{eprintln!("industry-tuf-freshness-audit: {e}");ExitCode::from(2)}
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn d(c:char)->String{std::iter::repeat_n(c,64).collect()}
    fn base()->(Value,Value,Value){
        let report=json!({
          "schema_version":VERIFIER_SCHEMA,"repository_id":"tuf:ieac-production",
          "root":{"version":3,"sha256":d('1'),"expires_at":"2026-12-01T00:00:00Z",
            "timestamp_keyset_sha256":d('a'),"snapshot_keyset_sha256":d('b')},
          "timestamp":{"version":14,"sha256":d('2'),"expires_at":"2026-11-03T00:00:00Z"},
          "snapshot":{"version":9,"sha256":d('3'),"expires_at":"2026-11-10T00:00:00Z"},
          "target_roles":[
            {"role_name":"ieac-channel-manifests","version":5,"metadata_sha256":d('4'),
             "expires_at":"2026-11-10T00:00:00Z","target_path":"channels/plant-17/manifest.json",
             "target_length":8192,"target_sha256":d('5')},
            {"role_name":"ieac-channel-checkpoints","version":8,"metadata_sha256":d('6'),
             "expires_at":"2026-11-10T00:00:00Z","target_path":"channels/plant-17/checkpoint.json",
             "target_length":2048,"target_sha256":d('7')},
            {"role_name":"ieac-channel-completeness-attestations","version":2,"metadata_sha256":d('8'),
             "expires_at":"2026-11-10T00:00:00Z","target_path":"channels/plant-17/completeness-attestation.json",
             "target_length":16384,"target_sha256":d('9')},
            {"role_name":"ieac-channel-completeness-reports","version":2,"metadata_sha256":d('b'),
             "expires_at":"2026-11-10T00:00:00Z","target_path":"channels/plant-17/completeness-report.json",
             "target_length":4096,"target_sha256":d('c')}
          ],
          "channel_binding":{
            "relying_party_id":"rp:industrial-assurance","channel_id":"channel:plant-17",
            "tenant_scope":"tenant:plant-17","purpose":"industrial_resilience_assurance",
            "ledger_id":"ledger:plant-17-evidence","sequence_namespace":"plant-17-events",
            "sequence_epoch":"epoch:2026-11-01","last_sequence":120,
            "channel_manifest":{"role_name":"ieac-channel-manifests","target_path":"channels/plant-17/manifest.json","target_length":8192,"target_sha256":d('5')},
            "checkpoint_state":{"role_name":"ieac-channel-checkpoints","target_path":"channels/plant-17/checkpoint.json","target_length":2048,"target_sha256":d('7')},
            "completeness_attestation":{"role_name":"ieac-channel-completeness-attestations","target_path":"channels/plant-17/completeness-attestation.json","target_length":16384,"target_sha256":d('9')},
            "availability_audit_report":{"role_name":"ieac-channel-completeness-reports","target_path":"channels/plant-17/completeness-report.json","target_length":4096,"target_sha256":d('c')},
            "epoch_transition":null
          },
          "root_update_chain":[],
          "verification_checks":{
            "timestamp_snapshot_binding_verified":true,"snapshot_targets_binding_verified":true,
            "delegation_authorization_verified":true,"target_hash_length_verified":true,
            "root_chain_verified":true,"timestamp_snapshot_key_rotation_recovery_verified":false
          },
          "evaluation_time":{
            "time_utc":"2026-11-02T09:30:00Z","uncertainty_before_seconds":2,
            "uncertainty_after_seconds":2,"clock_id":"clock:tuf-client",
            "verification_status":"verified","proof_sha256":d('8'),
            "verifier_build_sha256":d('a'),"verification_policy_sha256":d('b'),
            "trust_bundle_sha256":d('c')
          },
          "verifier":{"status":"verified","build_sha256":d('d'),
            "policy_sha256":d('e'),"trust_bundle_sha256":d('f')}
        });
        let policy=json!({
          "schema_version":POLICY_SCHEMA,"repository_id":"tuf:ieac-production",
          "relying_party_id":"rp:industrial-assurance","channel_id":"channel:plant-17",
          "tenant_scope":"tenant:plant-17","purpose":"industrial_resilience_assurance",
          "ledger_id":"ledger:plant-17-evidence","sequence_namespace":"plant-17-events",
          "manifest_target_path":"channels/plant-17/manifest.json",
          "manifest_target_role":"ieac-channel-manifests",
          "checkpoint_target_path":"channels/plant-17/checkpoint.json",
          "checkpoint_target_role":"ieac-channel-checkpoints",
          "completeness_attestation_target_path":"channels/plant-17/completeness-attestation.json",
          "completeness_attestation_target_role":"ieac-channel-completeness-attestations",
          "availability_report_target_path":"channels/plant-17/completeness-report.json",
          "availability_report_target_role":"ieac-channel-completeness-reports",
          "max_timestamp_expiry_horizon_seconds":172800,
          "time_verifier_build_sha256":d('a'),
          "time_verification_policy_sha256":d('b'),
          "time_verification_trust_bundle_sha256":d('c'),
          "tuf_verifier_build_sha256":d('d'),
          "tuf_verification_policy_sha256":d('e'),
          "tuf_trust_bundle_sha256":d('f'),
          "claim_ceiling":"tuf_freshness_and_anti_rollback_only","action_authority":"none"
        });
        let state=json!({
          "schema_version":STATE_SCHEMA,"state_id":"state:ieac-rp-plant17",
          "state_revision":18,"repository_id":"tuf:ieac-production",
          "relying_party_id":"rp:industrial-assurance","channel_id":"channel:plant-17",
          "tenant_scope":"tenant:plant-17","purpose":"industrial_resilience_assurance",
          "ledger_id":"ledger:plant-17-evidence","sequence_namespace":"plant-17-events",
          "sequence_epoch":"epoch:2026-11-01","last_sequence":118,
          "root":{"version":3,"sha256":d('1'),"expires_at":"2026-12-01T00:00:00Z",
            "timestamp_keyset_sha256":d('a'),"snapshot_keyset_sha256":d('b')},
          "timestamp":{"version":13,"sha256":d('9')},
          "snapshot":{"version":8,"sha256":d('8')},
          "target_roles":[
            {"role_name":"ieac-channel-manifests","version":4,"metadata_sha256":d('0')},
            {"role_name":"ieac-channel-checkpoints","version":7,"metadata_sha256":d('a')},
            {"role_name":"ieac-channel-completeness-attestations","version":1,"metadata_sha256":d('e')},
            {"role_name":"ieac-channel-completeness-reports","version":1,"metadata_sha256":d('f')}
          ],
          "manifest_sha256":d('0'),"checkpoint_state_sha256":d('a'),
          "completeness_attestation_sha256":d('e'),"availability_report_sha256":d('f')
        });
        (report,policy,state)
    }
    fn audit_base()->Result<AuditResult,String>{
        let (r,p,s)=base();
        let rb=serde_json::to_vec(&r).unwrap();
        let pb=serde_json::to_vec(&p).unwrap();
        let sb=serde_json::to_vec(&s).unwrap();
        audit(&r,&p,&s,&rb,&pb,&sb)
    }
    #[test]fn structurally_consistent_report_never_claims_current_or_advances_state(){
        let result=audit_base().unwrap();
        assert_eq!(result.status,"unverified");
        assert_eq!(result.structural_status,"consistent");
    }
    #[test]fn unknown_result_does_not_advance_high_water_state(){
        let (mut r,p,s)=base();
        r["timestamp"]["expires_at"]=json!("2026-11-02T09:29:00Z");
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        let result=audit(&r,&p,&s,&rb,&pb,&sb).unwrap();
        assert_eq!(result.status,"unverified");
        assert_eq!(result.structural_status,"unknown");
    }

    #[test]fn rejects_duplicate_json_members() {
        let parsed=parse_strict_json(br#"{"status":"verified","status":"failed"}"#,"duplicate-test");
        assert!(parsed.is_err());
    }
    #[test]fn accepts_well_formed_report_as_structurally_consistent_only(){
        let result=audit_base().unwrap();
        assert_eq!(result.status,"unverified");
        assert_eq!(result.structural_status,"consistent");
    }
    #[test]fn valid_root_rotation_accepts_contiguous_dual_threshold_chain(){
        let (mut r,p,s)=base();
        r["root"]["version"]=json!(4);
        r["root"]["sha256"]=json!(d('z'));
        r["root_update_chain"]=json!([{
            "from_version":3,"from_sha256":d('1'),
            "to_version":4,"to_sha256":d('z'),
            "old_threshold_verified":true,"new_threshold_verified":true
        }]);
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().structural_status,"consistent");
    }
    #[test]fn root_role_key_rotation_allows_tuf_timestamp_snapshot_recovery(){
        let (mut r,p,mut s)=base();
        r["root"]["version"]=json!(4);
        r["root"]["sha256"]=json!(d('z'));
        r["root"]["timestamp_keyset_sha256"]=json!(d('d'));
        r["root"]["snapshot_keyset_sha256"]=json!(d('e'));
        r["root_update_chain"]=json!([{
            "from_version":3,"from_sha256":d('1'),
            "to_version":4,"to_sha256":d('z'),
            "old_threshold_verified":true,"new_threshold_verified":true
        }]);
        r["timestamp"]["version"]=json!(1);
        r["timestamp"]["sha256"]=json!(d('a'));
        r["snapshot"]["version"]=json!(1);
        r["snapshot"]["sha256"]=json!(d('b'));
        r["verification_checks"]["timestamp_snapshot_key_rotation_recovery_verified"]=json!(true);
        s["root"]["timestamp_keyset_sha256"]=json!(d('b'));
        s["root"]["snapshot_keyset_sha256"]=json!(d('c'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().status,"current");
    }
    #[test]fn unverified_role_key_rotation_recovery_is_unknown(){
        let (mut r,p,mut s)=base();
        r["root"]["version"]=json!(4);
        r["root"]["sha256"]=json!(d('z'));
        r["root"]["timestamp_keyset_sha256"]=json!(d('d'));
        r["root"]["snapshot_keyset_sha256"]=json!(d('e'));
        r["root_update_chain"]=json!([{
            "from_version":3,"from_sha256":d('1'),
            "to_version":4,"to_sha256":d('z'),
            "old_threshold_verified":true,"new_threshold_verified":true
        }]);
        r["timestamp"]["version"]=json!(1);
        r["snapshot"]["version"]=json!(1);
        s["root"]["timestamp_keyset_sha256"]=json!(d('b'));
        s["root"]["snapshot_keyset_sha256"]=json!(d('c'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().structural_status,"unknown");
    }
    #[test]fn target_role_rollback_is_rejected(){
        let (mut r,p,s)=base();r["target_roles"][0]["version"]=json!(3);
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn excessive_timestamp_expiry_horizon_is_unknown(){
        let (mut r,p,s)=base();r["timestamp"]["expires_at"]=json!("2026-11-15T00:00:00Z");
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().status,"unknown");
    }

    #[test]fn timestamp_version_rollback_is_rejected(){
        let (mut r,p,s)=base();r["timestamp"]["version"]=json!(12);
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn same_version_changed_timestamp_digest_is_rejected(){
        let (mut r,p,s)=base();r["timestamp"]["version"]=json!(13);r["timestamp"]["sha256"]=json!(d('x'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn expired_timestamp_is_unknown_not_current(){
        let (mut r,p,s)=base();r["timestamp"]["expires_at"]=json!("2026-11-02T09:29:00Z");
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().status,"unknown");
    }
    #[test]fn wrong_tuf_trust_bundle_is_rejected(){
        let (mut r,p,s)=base();r["verifier"]["trust_bundle_sha256"]=json!(d('9'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn channel_sequence_rollback_is_rejected(){
        let (mut r,p,s)=base();r["channel_binding"]["last_sequence"]=json!(117);
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn same_sequence_checkpoint_equivocation_is_rejected(){
        let (mut r,p,s)=base();r["channel_binding"]["last_sequence"]=json!(118);
        r["channel_binding"]["checkpoint_state"]["target_sha256"]=json!(d('x'));
        r["target_roles"][1]["target_sha256"]=json!(d('x'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn completeness_attestation_target_substitution_is_rejected(){
        let (mut r,p,s)=base();
        r["channel_binding"]["completeness_attestation"]["target_sha256"]=json!(d('x'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn availability_audit_report_target_substitution_is_rejected(){
        let (mut r,p,s)=base();
        r["channel_binding"]["availability_audit_report"]["target_sha256"]=json!(d('x'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn completeness_target_same_role_version_cannot_change(){
        let (mut r,p,s)=base();
        r["channel_binding"]["completeness_attestation"]["target_sha256"]=json!(d('x'));
        r["target_roles"][2]["target_sha256"]=json!(d('x'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }

    #[test]fn missing_required_target_role_is_rejected(){
        let (mut r,p,s)=base();r["target_roles"].as_array_mut().unwrap().pop();
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn missing_root_rotation_chain_is_rejected(){
        let (mut r,p,s)=base();r["root"]["version"]=json!(4);r["root"]["sha256"]=json!(d('z'));
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn unverified_tuf_check_yields_unknown(){
        let (mut r,p,s)=base();r["verification_checks"]["delegation_authorization_verified"]=json!(false);
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert_eq!(audit(&r,&p,&s,&rb,&pb,&sb).unwrap().status,"unknown");
    }
    #[test]fn wrong_target_path_is_rejected(){
        let (mut r,p,s)=base();r["channel_binding"]["channel_manifest"]["target_path"]=json!("channels/plant-17/evil.json");
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
    #[test]fn missing_explicit_epoch_transition_is_rejected(){
        let (mut r,p,s)=base();r["channel_binding"]["sequence_epoch"]=json!("epoch:2026-11-02");
        let rb=serde_json::to_vec(&r).unwrap();let pb=serde_json::to_vec(&p).unwrap();let sb=serde_json::to_vec(&s).unwrap();
        assert!(audit(&r,&p,&s,&rb,&pb,&sb).is_err());
    }
}
