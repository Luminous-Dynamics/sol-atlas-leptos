// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Feature-gated native benchmark for the offline observation envelope.
//! All generated inputs remain synthetic; this is not a landscape-impact test.

#[path = "../landscape_observation.rs"]
mod landscape_observation;

use landscape_observation::{MAX_BATCH_OBSERVATIONS, normalize_batch, validate_batch};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{env, process::Command, time::{Instant, SystemTime, UNIX_EPOCH}};

const FIXTURE: &str = include_str!("../../fixtures/landscape/observation-batch-v0.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Profile { Unique, Dup1Pct, Dup10Pct, Dup100Pct, Conflict, OverLimit }

impl Profile {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "unique" => Ok(Self::Unique),
            "exact-1pct" => Ok(Self::Dup1Pct),
            "exact-10pct" => Ok(Self::Dup10Pct),
            "exact-100pct" => Ok(Self::Dup100Pct),
            "conflict" => Ok(Self::Conflict),
            "over-limit" => Ok(Self::OverLimit),
            _ => Err(format!("unknown profile {s:?}")),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Unique => "unique",
            Self::Dup1Pct => "exact-1pct",
            Self::Dup10Pct => "exact-10pct",
            Self::Dup100Pct => "exact-100pct",
            Self::Conflict => "conflict",
            Self::OverLimit => "over-limit",
        }
    }
    fn expected_valid(self) -> bool {
        !matches!(self, Self::Conflict | Self::OverLimit)
    }
}

struct Config {
    records: usize,
    iterations: usize,
    warmup: usize,
    profile: Profile,
    commit: String,
    machine: String,
    cpu: String,
    ram_mib: u64,
}

impl Config {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let (mut records, mut iterations, mut warmup) = (100usize, 25usize, 3usize);
        let mut profile = Profile::Unique;
        let mut commit = None;
        let mut machine = None;
        let mut cpu = None;
        let mut ram = None;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--records" => records = next_parse(&mut args, "--records")?,
                "--iterations" => iterations = next_parse(&mut args, "--iterations")?,
                "--warmup" => warmup = next_parse(&mut args, "--warmup")?,
                "--profile" => profile = Profile::parse(&args.next().ok_or("--profile needs a value")?)?,
                "--commit-sha" => commit = Some(args.next().ok_or("--commit-sha needs a value")?),
                "--machine-label" => machine = Some(args.next().ok_or("--machine-label needs a value")?),
                "--cpu-model" => cpu = Some(args.next().ok_or("--cpu-model needs a value")?),
                "--ram-mib" => ram = Some(next_parse(&mut args, "--ram-mib")?),
                other => return Err(format!("unknown argument {other:?}\n{}", Self::usage())),
            }
        }
        if records == 0 || records > MAX_BATCH_OBSERVATIONS + 1 || iterations == 0 || iterations > 100_000 || warmup > 100_000 {
            return Err("records must be 1..=10001; iterations 1..=100000; warmup <= 100000".into());
        }
        if profile == Profile::OverLimit { records = MAX_BATCH_OBSERVATIONS + 1; }
        if profile == Profile::Conflict && records < 2 { return Err("conflict profile requires at least 2 records".into()); }
        let commit = commit.ok_or("--commit-sha is required")?;
        if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("--commit-sha must be a 40- or 64-character Git SHA".into());
        }
        let machine = required(machine, "--machine-label")?;
        let cpu = required(cpu, "--cpu-model")?;
        let ram_mib = ram.ok_or("--ram-mib is required")?;
        if ram_mib == 0 { return Err("--ram-mib must be > 0".into()); }
        Ok(Self { records, iterations, warmup, profile, commit, machine, cpu, ram_mib })
    }
    fn usage() -> &'static str {
        "Usage: landscape-observation-bench --commit-sha SHA --machine-label LABEL --cpu-model MODEL --ram-mib MIB [--records N] [--iterations N] [--warmup N] [--profile unique|exact-1pct|exact-10pct|exact-100pct|conflict|over-limit]\nAll records are synthetic. Output is JSON including raw timing samples."
    }
}

fn required(v: Option<String>, name: &str) -> Result<String, String> {
    match v { Some(v) if !v.trim().is_empty() && v.len() <= 256 => Ok(v), _ => Err(format!("{name} must be supplied (max 256 chars")) }
}

fn next_parse<T: std::str::FromStr>(args: &mut impl Iterator<Item = String>, name: &str) -> Result<T, String>
where T::Err: std::fmt::Display {
    args.next().ok_or_else(|| format!("{name} needs a value"))?
        .parse().map_err(|e| format!("invalid {name}: {e}"))
}

fn sha(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn template_records() -> Result<Vec<Value>, String> {
    let batch: Value = serde_json::from_str(FIXTURE).map_err(|e| e.to_string())?;
    batch["observations"].as_array().cloned().ok_or_else(|| "fixture observations are missing".into())
}

fn generated_record(template: &Value, i: usize) -> Value {
    let mut record = template.clone();
    let suffix = format!("{i:08}");
    record["observation_id"] = json!(format!("obs-bench-generated-{suffix}"));
    record["source_event_id"] = json!(format!("bench-event-{suffix}"));
    record["project_ref"]["feature_id"] = json!(format!("feature-{:04}", i + 1));
    let payload = sha(format!("landscape-observation-bench-v0:0x4f425356:raw:{i}").as_bytes());
    record["source"]["source_id"] = json!(format!("bench-source-{suffix}"));
    record["source"]["payload_sha256"] = json!(payload);
    record["source"]["source_version"] = json!("landscape-observation-bench-v0");
    let delta = (i as u64).saturating_mul(1000);
    for k in ["start_unix_ms", "end_unix_ms"] {
        if let Some(n) = record["phenomenon_time"][k].as_u64() {
            record["phenomenon_time"][k] = json!(n.saturating_add(delta));
        }
    }
    if let Some(n) = record["result_time_unix_ms"].as_u64() {
        record["result_time_unix_ms"] = json!(n.saturating_add(delta));
    }
    if record["result"]["kind"] == "quantity" {
        if let Some(n) = record["result"]["numeric_value"].as_f64() {
            record["result"]["numeric_value"] = json!(n + (i % 101) as f64 / 100.0);
        }
    }
    if let Some(steps) = record["transformations"].as_array_mut() {
        let mut previous = payload;
        for (step, item) in steps.iter_mut().enumerate() {
            item["input_sha256"] = json!(previous);
            let next = sha(format!("landscape-observation-bench-v0:output:{i}:{step}").as_bytes());
            item["output_sha256"] = json!(next);
            previous = next;
            if let Some(t) = item["performed_at_unix_ms"].as_u64() {
                item["performed_at_unix_ms"] = json!(t.saturating_add(delta));
            }
        }
    }
    record
}

fn generate(config: &Config) -> Result<Value, String> {
    let templates = template_records()?;
    if templates.is_empty() { return Err("fixture has no observation records".into()); }
    let duplicate_count = match config.profile {
        Profile::Dup1Pct => config.records.saturating_mul(100).div_ceil(10_000).min(config.records - 1),
        Profile::Dup10Pct => config.records.saturating_mul(1_000).div_ceil(10_000).min(config.records - 1),
        Profile::Dup100Pct => config.records.saturating_sub(1),
        _ => 0,
    };
    let unique_count = config.records.saturating_sub(duplicate_count);
    let mut observations = Vec::with_capacity(config.records);
    for i in 0..unique_count { observations.push(generated_record(&templates[i % templates.len()], i)); }
    if duplicate_count > 0 {
        let first = observations.first().ok_or("duplicate profile has no canonical record")?.clone();
        observations.extend(std::iter::repeat_n(first, duplicate_count));
    }
    if config.profile == Profile::Conflict {
        observations.clear();
        for i in 0..config.records - 1 { observations.push(generated_record(&templates[0], i)); }
        let mut changed = observations[0].clone();
        changed["result"]["numeric_value"] = json!(changed["result"]["numeric_value"].as_f64().unwrap_or(0.0) + 1.0);
        observations.push(changed);
    }
    if config.profile == Profile::OverLimit {
        observations.clear();
        for i in 0..config.records { observations.push(generated_record(&templates[i % templates.len()], i)); }
    }
    let created = observations.iter()
        .filter_map(|o| o["result_time_unix_ms"].as_u64()).max().unwrap_or(0).saturating_add(1000);
    Ok(json!({
        "schema":"luminous.landscape-observation-batch.v0",
        "schema_uri":"https://luminousdynamics.io/schemas/landscape-observation-batch-v0.schema.json",
        "batch_id":format!("obs-batch-landscape-bench-{}", config.profile.label()),
        "created_at_unix_ms":created,
        "observations":observations
    }))
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    if samples.is_empty() { return 0; }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(p).div_ceil(100).max(1);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn summary(samples: &[u128]) -> Value {
    json!({
        "sample_count":samples.len(),
        "p50_ns":percentile(samples, 50),
        "p95_ns":percentile(samples, 95),
        "p99_ns":percentile(samples, 99),
        "raw_samples_ns":samples
    })
}

fn measure<T>(iterations: usize, warmup: usize, mut f: impl FnMut() -> T) -> (Vec<u128>, Vec<T>) {
    for _ in 0..warmup { std::hint::black_box(f()); }
    let mut times = Vec::with_capacity(iterations);
    let mut values = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let value = f();
        times.push(start.elapsed().as_nanos());
        values.push(value);
    }
    (times, values)
}

fn rustc_version() -> Result<String, String> {
    let output = Command::new("rustc").arg("--version").output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Err("rustc --version failed".into()); }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().try_into().unwrap_or(u64::MAX)
}

fn run(config: Config) -> Result<Value, String> {
    let batch = generate(&config)?;
    let input = serde_json::to_vec(&batch).map_err(|e| e.to_string())?;
    let input_sha = sha(&input);
    let expected = config.profile.expected_valid();
    let (v_times, v_results) = measure(config.iterations, config.warmup, || validate_batch(&batch));
    let (n_times, n_results) = measure(config.iterations, config.warmup, || normalize_batch(&batch));
    let v_mismatch = v_results.iter().filter(|r| r.is_ok() != expected).count();
    let n_mismatch = n_results.iter().filter(|r| r.is_ok() != expected).count();
    let last_validation = v_results.last().map(|r| match r {
        Ok(v) => json!({"ok":true,"input_records":v.input_records,"unique_observations":v.unique_observations,"duplicates":v.identical_duplicate_records}),
        Err(e) => json!({"ok":false,"errors":e}),
    }).unwrap_or(Value::Null);
    let normalized = n_results.last().and_then(|r| r.as_ref().ok()).and_then(|(v,_)| serde_json::to_vec(v).ok());
    let normalized_sha = normalized.as_deref().map(sha);
    let mut sorted_v = v_times.clone();
    sorted_v.sort_unstable();
    let p50 = percentile(&sorted_v,50);
    let throughput = if p50 == 0 { Value::Null } else { json!(config.records as f64 / (p50 as f64 / 1_000_000_000.0)) };
    let raw_samples = json!({"validate_ns":v_times,"normalize_ns":n_times,"validation_mismatches":v_mismatch,"normalization_mismatches":n_mismatch});
    let raw_samples_sha = sha(&serde_json::to_vec(&raw_samples).map_err(|e| e.to_string())?);
    let correct = v_mismatch == 0 && n_mismatch == 0;
    Ok(json!({
        "schema":"luminous.landscape-observation-benchmark-result.v0",
        "status":if correct {"correctness-pass"} else {"correctness-fail"},
        "measured_at_unix_ms":unix_ms(),
        "protocol":"landscape-observation-ingest-benchmark-v0",
        "generator":{"version":"landscape-observation-bench-v0","seed":"0x4f425356","profile":config.profile.label()},
        "input":{"record_count":config.records,"byte_count":input.len(),"sha256":input_sha,"expected_valid":expected,"last_validation":last_validation,"normalized_byte_count":normalized.as_ref().map(Vec::len),"normalized_sha256":normalized_sha},
        "timing":{"warmup_iterations":config.warmup,"measured_iterations":config.iterations,"validation":summary(&v_times),"normalization":summary(&n_times),"validation_records_per_second_at_p50":throughput},
        "correctness":{"validation_mismatches":v_mismatch,"normalization_mismatches":n_mismatch,"no_false_accepts_or_false_rejects":correct},
        "raw_samples_sha256":raw_samples_sha,
        "metadata":{"commit_sha":config.commit,"machine_label":config.machine,"cpu_model":config.cpu,"ram_mib":config.ram_mib,"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"rustc":rustc_version()?,"input_resident_in_memory":true,"network_used":false,"cache_claim":"input remains resident in memory; OS cold-cache state is not measured"}
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") { println!("{}", Config::usage()); return; }
    let config = match Config::parse(args) {
        Ok(config) => config,
        Err(error) => { eprintln!("{error}\n{}", Config::usage()); std::process::exit(2); }
    };
    match run(config) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{output}"),
            Err(error) => { eprintln!("JSON serialization failed: {error}"); std::process::exit(3); }
        },
        Err(error) => { eprintln!("{error}"); std::process::exit(4); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(records: usize, profile: Profile) -> Config {
        Config { records, iterations: 2, warmup: 0, profile, commit:"a".repeat(40), machine:"test-machine".into(), cpu:"synthetic-test-cpu".into(), ram_mib:1024 }
    }

    #[test]
    fn unique_generator_is_valid_and_repeatable() {
        let config=cfg(20,Profile::Unique);
        let a=generate(&config).unwrap();
        let b=generate(&config).unwrap();
        assert_eq!(a,b);
        assert!(validate_batch(&a).is_ok());
    }

    #[test]
    fn exact_duplicates_normalize_deterministically() {
        let batch=generate(&cfg(20,Profile::Dup100Pct)).unwrap();
        let summary=validate_batch(&batch).unwrap();
        assert_eq!(summary.input_records,20);
        assert_eq!(summary.unique_observations,1);
        assert_eq!(summary.identical_duplicate_records,19);
        let (normalized,_) = normalize_batch(&batch).unwrap();
        assert_eq!(normalized["observations"].as_array().unwrap().len(),1);
    }

    #[test]
    fn conflicting_id_is_rejected_by_both_paths() {
        let batch=generate(&cfg(5,Profile::Conflict)).unwrap();
        assert!(validate_batch(&batch).is_err());
        assert!(normalize_batch(&batch).is_err());
    }

    #[test]
    fn over_limit_profile_rejected_early() {
        let batch=generate(&cfg(MAX_BATCH_OBSERVATIONS+1,Profile::OverLimit)).unwrap();
        assert!(validate_batch(&batch).unwrap_err().iter().any(|e|e.contains("exceeds limit")));
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        assert_eq!(percentile(&[10,20,30,40,50],50),30);
        assert_eq!(percentile(&[10,20,30,40,50],95),50);
        assert_eq!(percentile(&[],95),0);
    }
}
