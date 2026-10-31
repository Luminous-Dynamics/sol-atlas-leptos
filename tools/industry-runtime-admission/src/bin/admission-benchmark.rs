use industry_assurance_validator::validate_profile;
use industry_runtime_admission::{
    evaluate_admission, independent_check, CHECKER_VERSION, EVALUATOR_VERSION,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Instant;

const FAMILIES: &[(&str, &str, &[&str], bool)] = &[
    ("positive_control", "admit_descriptive", &[], false),
    ("exact_min_boundary", "admit_descriptive", &[], false),
    ("exact_max_boundary", "admit_descriptive", &[], false),
    ("stale_measurement", "abstain", &["MEASUREMENT_STALE"], true),
    ("future_measurement", "reject", &["FUTURE_DATED_MEASUREMENT"], true),
    ("missing_measurement", "abstain", &["CONDITION_MEASUREMENT_MISSING"], true),
    ("duplicate_measurement", "reject", &["DUPLICATE_CONDITION_MEASUREMENT"], true),
    ("interval_crossing", "abstain", &["MEASUREMENT_INTERVAL_CROSSES_ENVELOPE"], true),
    ("out_of_range", "reject", &["VALUE_OUTSIDE_OPERATING_ENVELOPE"], true),
    ("just_below_min_boundary", "reject", &["VALUE_OUTSIDE_OPERATING_ENVELOPE"], true),
    ("just_above_max_boundary", "reject", &["VALUE_OUTSIDE_OPERATING_ENVELOPE"], true),
    ("unit_mismatch", "reject", &["UNIT_MISMATCH"], true),
    ("identity_drift", "reject", &["RUNTIME_IDENTITY_BINDING_INVALID"], true),
    ("profile_expired", "reject", &["PROFILE_OUTSIDE_VALIDITY_WINDOW"], true),
    ("profile_pin_revoked", "reject", &["PROFILE_PIN_REVOKED"], true),
    ("source_revoked", "reject", &["ARTIFACT_REVOKED"], true),
    ("authorization_unavailable", "abstain", &["REVOCATION_STATE_UNAVAILABLE"], true),
    ("authority_escalation", "reject", &["DECISION_TIER_NOT_ADMITTED"], true),
    ("stale_and_source_revoked", "reject", &["ARTIFACT_REVOKED", "MEASUREMENT_STALE"], true),
];

struct Rng { state: u64 }

impl Rng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}
fn digest_value(value: &Value) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("JSON values serialize"))
}
fn refresh_digests(profile: &Value, request: &mut Value) {
    request["profile_sha256"] = json!(digest_value(profile));
    request["snapshot_sha256"] = json!(digest_value(&request["snapshot"]));
    request["revocation_snapshot_sha256"] = json!(digest_value(&request["revocation_snapshot"]));
}
fn refresh_pin(profile: &Value, pin: &mut Value) {
    pin["profile_id"] = profile["profile_id"].clone();
    pin["profile_sha256"] = json!(digest_value(profile));
}
fn read_fixture(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON in {path}: {e}"))
}

fn vertical_spec(vertical: &str) -> Result<(&'static str, &'static str, f64, f64, f64, f64, f64), String> {
    match vertical {
        "telecommunications" => Ok(("site_backup_usable_energy", "kWh", 0.0, 20.0, 12.0, 11.0, 13.0)),
        "water_food_energy" => Ok(("water_pressure", "kPa", 100.0, 500.0, 300.0, 295.0, 305.0)),
        "industrial_maintenance" => Ok(("bearing_vibration", "mm/s", 0.0, 8.0, 2.5, 2.0, 3.0)),
        "buildings" => Ok(("indoor_temperature", "degC", 18.0, 28.0, 22.0, 21.5, 22.5)),
        other => Err(format!("unsupported benchmark.vertical {other:?}")),
    }
}

fn build_vertical_fixture(
    vertical: &str,
    profile: &mut Value,
    pin: &mut Value,
    request: &mut Value,
) -> Result<(), String> {
    let (variable, unit, minimum, maximum, value, lower, upper) = vertical_spec(vertical)?;
    let slug = vertical.replace('_', "-");
    let asset = format!("synthetic-{slug}-asset-001");
    let sensor = format!("sensor-{slug}-001");
    let capability = format!("{slug}-bounded-admission");
    let decision = format!("{slug}-advisory-review");
    let model = format!("{slug}-model-v0.1");
    let topology_sha = "d4".repeat(32);

    profile["profile_id"] = json!(format!("synthetic-{slug}-runtime-admission-v0.1"));
    profile["claim"]["capability_id"] = json!(capability);
    profile["claim"]["instance_id"] = json!(asset);
    profile["claim"]["decision_id"] = json!(decision);
    profile["claim"]["model_ids"] = json!([model]);
    profile["operating_envelope"]["conditions"] = json!([{
        "variable": variable,
        "unit": unit,
        "minimum": minimum,
        "maximum": maximum,
        "max_age_seconds": 3600,
        "support": "Synthetic stratified benchmark condition only; no physical validation is implied.",
        "evidence_state": "measured",
        "source_artifact_ids": [sensor]
    }]);

    let bindings = profile["artifact_bindings"].as_array_mut()
        .ok_or("profile artifact_bindings must be an array")?;
    for binding in bindings.iter_mut() {
        match binding["artifact_id"].as_str() {
            Some("sensor-battery-001") => binding["artifact_id"] = json!(sensor),
            Some("topology-001") => binding["content_sha256"] = json!(topology_sha),
            _ => {}
        }
    }
    let sensor_sha = bindings.iter()
        .find(|b| b["artifact_id"].as_str() == Some(sensor.as_str()))
        .and_then(|b| b["content_sha256"].as_str())
        .ok_or("vertical sensor binding is missing")?.to_owned();

    request["profile_id"] = profile["profile_id"].clone();
    request["capability_id"] = json!(capability);
    request["instance_id"] = json!(asset);
    request["decision_id"] = json!(decision);
    request["model_id"] = json!(model);
    request["topology_sha256"] = json!(topology_sha);
    request["asset_id"] = json!(asset);
    request["snapshot"]["asset_id"] = request["asset_id"].clone();
    request["snapshot"]["topology_sha256"] = json!(topology_sha);
    request["snapshot"]["conditions"][0]["variable"] = json!(variable);
    request["snapshot"]["conditions"][0]["unit"] = json!(unit);
    request["snapshot"]["conditions"][0]["value"] = json!(value);
    request["snapshot"]["conditions"][0]["lower"] = json!(lower);
    request["snapshot"]["conditions"][0]["upper"] = json!(upper);
    request["snapshot"]["conditions"][0]["evidence_artifact_id"] = json!(sensor);
    request["snapshot"]["conditions"][0]["evidence_sha256"] = json!(sensor_sha);

    refresh_pin(profile, pin);
    refresh_digests(profile, request);
    Ok(())
}

fn apply_family(
    family: &str,
    trial: usize,
    rng: &mut Rng,
    profile: &mut Value,
    pin: &mut Value,
    request: &mut Value,
) {
    let family_slug = family.replace('_', "-");
    request["request_id"] = json!(format!("bench-{family_slug}-{trial}"));
    request["snapshot_id"] = json!(format!("snapshot-{family_slug}-{trial}"));
    request["snapshot"]["snapshot_id"] = request["snapshot_id"].clone();

    let minimum = profile["operating_envelope"]["conditions"][0]["minimum"].as_f64().unwrap_or(0.0);
    let maximum = profile["operating_envelope"]["conditions"][0]["maximum"].as_f64().unwrap_or(1.0);
    let span = (maximum - minimum).max(1e-9);
    let value = minimum + span * (0.25 + 0.5 * rng.unit_f64());

    if family == "missing_measurement" {
        request["snapshot"]["conditions"] = json!([]);
    } else if family == "duplicate_measurement" {
        let first = request["snapshot"]["conditions"][0].clone();
        request["snapshot"]["conditions"] = json!([first.clone(), first]);
    } else {
        {
            let condition = &mut request["snapshot"]["conditions"][0];
            condition["value"] = json!(value);
            condition["lower"] = json!(value - span * 0.02);
            condition["upper"] = json!(value + span * 0.02);
            match family {
                "positive_control" => condition["observed_at"] = json!("2026-10-31T11:30:00Z"),
                "stale_measurement" => condition["observed_at"] = json!("2026-10-31T00:00:00Z"),
                "future_measurement" => condition["observed_at"] = json!("2026-10-31T12:00:01Z"),
                "stale_and_source_revoked" => condition["observed_at"] = json!("2026-10-31T00:00:00Z"),
                "exact_min_boundary" => {
                    condition["value"] = json!(minimum);
                    condition["lower"] = json!(minimum);
                    condition["upper"] = json!(minimum);
                }
                "exact_max_boundary" => {
                    condition["value"] = json!(maximum);
                    condition["lower"] = json!(maximum);
                    condition["upper"] = json!(maximum);
                }
                "just_below_min_boundary" => {
                    let boundary_value = minimum - span * 1e-9;
                    condition["value"] = json!(boundary_value);
                    condition["lower"] = json!(boundary_value);
                    condition["upper"] = json!(boundary_value);
                }
                "just_above_max_boundary" => {
                    let boundary_value = maximum + span * 1e-9;
                    condition["value"] = json!(boundary_value);
                    condition["lower"] = json!(boundary_value);
                    condition["upper"] = json!(boundary_value);
                }
                "interval_crossing" => {
                    condition["value"] = json!(minimum + span * 0.25);
                    condition["lower"] = json!(minimum - span * 0.01);
                    condition["upper"] = json!(minimum + span * 0.30);
                }
                "out_of_range" => {
                    condition["value"] = json!(maximum + span * 0.10);
                    condition["lower"] = json!(maximum + span * 0.08);
                    condition["upper"] = json!(maximum + span * 0.12);
                }
                "unit_mismatch" => condition["unit"] = json!("__wrong_unit__"),
                _ => {}
            }
        }
        match family {
            "identity_drift" => request["model_sha256"] = json!("ff".repeat(32)),
            "profile_expired" => {
                profile["validity"]["review_by"] = json!("2026-10-31T11:59:59Z");
                refresh_pin(profile, pin);
                request["profile_sha256"] = json!(digest_value(profile));
            }
            "profile_pin_revoked" => pin["status"] = json!("revoked"),
            "source_revoked" | "stale_and_source_revoked" => {
                let source_id = request["snapshot"]["conditions"][0]["evidence_artifact_id"].clone();
                request["revocation_snapshot"]["revoked_artifact_ids"] = json!([source_id]);
            }
            "authorization_unavailable" => request["revocation_snapshot"]["status"] = json!("unavailable"),
            "authority_escalation" => {
                request["decision_tier"] = json!("D2_delegated_planning");
                request["action"] = json!("prepare_plan");
            }
            _ => {}
        }
    }
    refresh_digests(profile, request);
}

fn reason_codes_match(report: &Value, expected: &[&str]) -> bool {
    let Some(codes) = report["reason_codes"].as_array() else {
        return false;
    };
    let mut actual = codes.iter().filter_map(Value::as_str).map(str::to_owned).collect::<Vec<_>>();
    let mut expected = expected.iter().map(|code| (*code).to_owned()).collect::<Vec<_>>();
    actual.sort_unstable();
    expected.sort_unstable();
    actual == expected
}

fn percentile(values: &[f64], quantile: f64) -> Option<f64> {
    if values.is_empty() { return None; }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = ((quantile * sorted.len() as f64).ceil() as usize).max(1);
    sorted.get(rank - 1).copied()
}
fn log_add_exp(a: f64, b: f64) -> f64 {
    if a.is_infinite() && a.is_sign_negative() { return b; }
    if b.is_infinite() && b.is_sign_negative() { return a; }
    let high = a.max(b);
    high + ((a - high).exp() + (b - high).exp()).ln()
}
fn binomial_cdf(k: usize, n: usize, p: f64) -> f64 {
    if p <= 0.0 { return 1.0; }
    if p >= 1.0 { return if k >= n { 1.0 } else { 0.0 }; }
    if k >= n { return 1.0; }
    let lp = p.ln();
    let lq = (-p).ln_1p();
    let mut log_choose = 0.0;
    let mut log_sum = f64::NEG_INFINITY;
    for i in 0..=k {
        if i > 0 { log_choose += ((n - i + 1) as f64 / i as f64).ln(); }
        let term = log_choose + i as f64 * lp + (n - i) as f64 * lq;
        log_sum = log_add_exp(log_sum, term);
    }
    log_sum.exp().clamp(0.0, 1.0)
}
fn one_sided_exact_upper(events: usize, trials: usize, alpha: f64) -> f64 {
    if trials == 0 || events >= trials { return 1.0; }
    if events == 0 { return 1.0 - alpha.powf(1.0 / trials as f64); }
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..64 {
        let mid = (low + high) * 0.5;
        if binomial_cdf(events, trials, mid) > alpha { low = mid; } else { high = mid; }
    }
    high
}

fn validate_manifest(manifest: &Value) -> Result<(&str, usize, u64), String> {
    if manifest["schema_version"].as_str() != Some("luminous.industrial-benchmark-run-manifest.v0.1") {
        return Err("manifest is not an IBAP v0.1 run manifest".to_owned());
    }
    if manifest["benchmark"]["evaluation_mode"].as_str() != Some("synthetic_fixture") {
        return Err("this harness build accepts synthetic_fixture manifests only".to_owned());
    }
    if manifest["freeze"]["pre_registered_before_execution"].as_bool() != Some(true) {
        return Err("benchmark manifest is not marked pre-registered before execution".to_owned());
    }
    let methods = manifest["methods"].as_array().ok_or("manifest.methods must be an array")?;
    let has_candidate = methods.iter().any(|m| m["role"].as_str() == Some("candidate"));
    let has_baseline = methods.iter().any(|m| {
        matches!(m["role"].as_str(), Some("null_baseline" | "incumbent" | "domain_reference"))
    });
    if !has_candidate || !has_baseline {
        return Err("manifest must declare one candidate and at least one baseline".to_owned());
    }
    if methods.iter().filter(|m| m["role"].as_str() == Some("candidate")).count() != 1 {
        return Err("manifest must declare exactly one candidate method".to_owned());
    }
    if manifest["claim_ceiling"].as_str() != Some("synthetic_only") {
        return Err("this runner only accepts synthetic_only claim ceilings".to_owned());
    }
    if manifest["comparability"]["state"].as_str() != Some("matched") {
        return Err("benchmark methods are not declared comparable".to_owned());
    }
    let replication = &manifest["replication"];
    let replicate_count = replication["replicate_count"].as_u64()
        .ok_or("manifest.replication.replicate_count is required")?;
    let seeds = replication["seeds"].as_array()
        .ok_or("manifest.replication.seeds must be an array")?;
    if replicate_count != 1 || seeds.len() != 1 {
        return Err("v0.1 runner requires exactly one preregistered replicate and one seed".to_owned());
    }
    let frozen_seed = seeds[0].as_u64().ok_or("manifest seed must be an unsigned integer")?;
    let frozen_trials = replication["trials_per_stratum"].as_u64()
        .filter(|value| *value > 0 && *value <= 1_000_000)
        .ok_or("manifest.replication.trials_per_stratum must be in 1..=1000000")? as usize;
    let vertical = manifest["benchmark"]["vertical"].as_str()
        .ok_or("manifest.benchmark.vertical is required")?;
    vertical_spec(vertical)?;
    Ok((vertical, frozen_trials, frozen_seed))
}

fn validate_cli_overrides(
    requested_trials: Option<usize>,
    requested_seed: Option<u64>,
    frozen_trials: usize,
    frozen_seed: u64,
) -> Result<(), String> {
    if requested_trials.is_some_and(|value| value != frozen_trials) {
        return Err(format!(
            "CLI trial count {:?} differs from preregistered trials_per_stratum {frozen_trials}",
            requested_trials
        ));
    }
    if requested_seed.is_some_and(|value| value != frozen_seed) {
        return Err(format!(
            "CLI seed {:?} differs from preregistered seed {frozen_seed}",
            requested_seed
        ));
    }
    Ok(())
}

fn parse_args() -> Result<(String, Option<usize>, Option<u64>, Option<String>), String> {
    let mut manifest = None;
    let mut output = None;
    let mut trials = None;
    let mut seed = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().ok_or_else(|| format!("missing value after {arg}"))?;
        match arg.as_str() {
            "--manifest" => manifest = Some(value),
            "--output" => output = Some(value),
            "--trials-per-family" => {
                let parsed = value.parse::<usize>().map_err(|_| "trials-per-family must be a positive integer")?;
                if parsed == 0 || parsed > 1_000_000 {
                    return Err("trials-per-family must be in 1..=1000000".to_owned());
                }
                trials = Some(parsed);
            }
            "--seed" => {
                seed = Some(if let Some(hex) = value.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16).map_err(|_| "seed hex value is invalid")?
                } else {
                    value.parse().map_err(|_| "seed must be decimal or 0x-prefixed hexadecimal")?
                });
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    Ok((
        manifest.ok_or("required argument: --manifest <frozen-ibap-manifest.json>")?,
        trials, seed, output,
    ))
}

fn write_new(path: &str, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| format!("refusing to overwrite or cannot create {path}: {e}"))?;
    file.write_all(bytes).map_err(|e| format!("cannot write {path}: {e}"))?;
    file.write_all(b"\n").map_err(|e| format!("cannot finish {path}: {e}"))?;
    file.sync_all().map_err(|e| format!("cannot sync {path}: {e}"))?;
    Ok(())
}

fn run() -> Result<(Value, bool), String> {
    let (manifest_path, requested_trials, requested_seed, output_path) = parse_args()?;
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|e| format!("cannot read manifest {manifest_path}: {e}"))?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("invalid manifest JSON: {e}"))?;
    let (vertical, trials_per_family, seed) = validate_manifest(&manifest)?;
    validate_cli_overrides(requested_trials, requested_seed, trials_per_family, seed)?;
    let manifest_sha256 = digest_bytes(&manifest_bytes);

    let mut profile = read_fixture("schemas/examples/industry-runtime-admission-profile-synthetic-v0.1.json")?;
    let mut pin = read_fixture("schemas/examples/industry-runtime-profile-pin-synthetic-v0.1.json")?;
    let mut request = read_fixture("schemas/examples/industry-runtime-admission-request-synthetic-v0.1.json")?;
    build_vertical_fixture(vertical, &mut profile, &mut pin, &mut request)?;

    let methods = manifest["methods"].as_array().unwrap();
    let candidate_method = methods.iter().find(|m| m["role"].as_str() == Some("candidate")).unwrap();
    let baseline_method = methods.iter().find(|m| {
        matches!(m["role"].as_str(), Some("null_baseline" | "incumbent" | "domain_reference"))
    }).unwrap();

    let mut rng = Rng::new(seed);
    let run_started = Instant::now();
    let mut strata = Vec::new();
    let mut negative_trials_total = 0usize;
    let mut negative_rejects_total = 0usize;
    let mut negative_abstentions_total = 0usize;
    let mut false_admissions_total = 0usize;
    let mut positive_trials_total = 0usize;
    let mut positive_admissions_total = 0usize;
    let mut positive_abstentions_total = 0usize;
    let mut positive_rejections_total = 0usize;
    let mut checker_disagreements_total = 0usize;
    let mut tamper_checks_total = 0usize;
    let mut tamper_detected_total = 0usize;
    let mut all_latencies_ms = Vec::new();
    let mut replay_checks = 0usize;
    let mut replay_mismatches = 0usize;
    let mut disposition_mismatches_total = 0usize;
    let mut reason_code_misses_total = 0usize;
    let mut corpus_hasher = Sha256::new();
    let mut family_simultaneous_upper_bounds = Vec::new();
    let mut family_false_admission_rates = Vec::new();
    let mut static_baseline_counterfactual_acceptances_total = 0usize;

    for (family, expected, expected_reason, is_negative) in FAMILIES {
        let mut false_admissions = 0usize;
        let mut false_rejections = 0usize;
        let mut negative_rejects = 0usize;
        let mut negative_abstentions = 0usize;
        let mut positive_admissions = 0usize;
        let mut positive_abstentions = 0usize;
        let mut positive_rejections = 0usize;
        let mut expected_disposition_matches = 0usize;
        let mut checker_disagreements = 0usize;
        let mut reason_code_hits = 0usize;
        let mut tamper_detected = 0usize;
        let mut static_profile_valid_trials = 0usize;
        let mut static_false_admissions = 0usize;
        let mut latencies_ms = Vec::with_capacity(trials_per_family);

        for trial in 0..trials_per_family {
            let mut case_profile = profile.clone();
            let mut case_pin = pin.clone();
            let mut case_request = request.clone();
            apply_family(family, trial, &mut rng, &mut case_profile, &mut case_pin, &mut case_request);
            for value in [&case_profile, &case_pin, &case_request] {
                let bytes = serde_json::to_vec(value).expect("benchmark inputs serialize");
                corpus_hasher.update((bytes.len() as u64).to_le_bytes());
                corpus_hasher.update(bytes);
            }
            let static_valid_for_case = validate_profile(&case_profile).is_empty();
            if static_valid_for_case {
                static_profile_valid_trials += 1;
                if *is_negative {
                    static_false_admissions += 1;
                    static_baseline_counterfactual_acceptances_total += 1;
                }
            }

            let started = Instant::now();
            let report = evaluate_admission(&case_profile, &case_pin, &case_request);
            let checker_errors = independent_check(&case_profile, &case_pin, &case_request, &report);
            let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
            latencies_ms.push(latency_ms);
            all_latencies_ms.push(latency_ms);

            let actual = report["admission"].as_str().unwrap_or("invalid");
            let admitted = actual.starts_with("admit_");
            if actual == *expected { expected_disposition_matches += 1; }
            if *is_negative {
                negative_trials_total += 1;
                if admitted {
                    false_admissions += 1;
                    false_admissions_total += 1;
                } else if actual == "reject" {
                    negative_rejects += 1;
                    negative_rejects_total += 1;
                } else if actual == "abstain" {
                    negative_abstentions += 1;
                    negative_abstentions_total += 1;
                }
            } else {
                positive_trials_total += 1;
                if admitted {
                    positive_admissions += 1;
                    positive_admissions_total += 1;
                } else if actual == "abstain" {
                    false_rejections += 1;
                    positive_abstentions += 1;
                    positive_abstentions_total += 1;
                } else {
                    false_rejections += 1;
                    positive_rejections += 1;
                    positive_rejections_total += 1;
                }
            }
            if !checker_errors.is_empty() {
                checker_disagreements += 1;
                checker_disagreements_total += 1;
            }

            if reason_codes_match(&report, expected_reason) {
                reason_code_hits += 1;
            }

            let mut tampered = report.clone();
            tampered["admission"] = json!(if admitted { "reject" } else { "admit_descriptive" });
            tamper_checks_total += 1;
            if !independent_check(&case_profile, &case_pin, &case_request, &tampered).is_empty() {
                tamper_detected += 1;
                tamper_detected_total += 1;
            }

            if trial < 3 {
                replay_checks += 1;
                let replay = evaluate_admission(&case_profile, &case_pin, &case_request);
                if serde_json::to_vec(&replay).ok() != serde_json::to_vec(&report).ok() {
                    replay_mismatches += 1;
                }
            }
        }

        if expected_disposition_matches != trials_per_family {
            disposition_mismatches_total += trials_per_family - expected_disposition_matches;
        }
        if reason_code_hits != trials_per_family {
            reason_code_misses_total += trials_per_family - reason_code_hits;
        }
        if *is_negative {
            family_false_admission_rates.push(false_admissions as f64 / trials_per_family as f64);
        }
        let upper_bound = if *is_negative {
            let negative_family_count = FAMILIES.iter().filter(|(_, _, _, negative)| *negative).count();
            let simultaneous_alpha = 0.05 / negative_family_count as f64;
            let simultaneous_bound = one_sided_exact_upper(false_admissions, trials_per_family, simultaneous_alpha);
            family_simultaneous_upper_bounds.push(simultaneous_bound);
            Some(one_sided_exact_upper(false_admissions, trials_per_family, 0.05))
        } else { None };

        strata.push(json!({
            "vertical": vertical,
            "failure_family": family,
            "expected_admission": expected,
            "trials": trials_per_family,
            "negative_control": is_negative,
            "candidate": {
                "false_admissions": false_admissions,
                "false_admission_rate": if *is_negative { json!(false_admissions as f64 / trials_per_family as f64) } else { Value::Null },
                "one_sided_exact_95_percent_false_admission_upper_bound": upper_bound,
                "false_rejections_on_positive_controls": false_rejections,
                "positive_admissions": positive_admissions,
                "positive_abstentions": positive_abstentions,
                "positive_rejections": positive_rejections,
                "negative_rejects": negative_rejects,
                "negative_abstentions": negative_abstentions,
                "expected_disposition_matches": expected_disposition_matches,
                "independent_checker_disagreements": checker_disagreements,
                "expected_reason_code_hits": reason_code_hits
            },
            "static_only_profile_baseline": {
                "profile_valid_trials": static_profile_valid_trials,
                "does_not_evaluate_request_freshness_or_runtime_identity": true,
                "would_counterfactually_accept_if_profile_validation_were_treated_as_runtime_authorization": static_false_admissions
            },
            "report_tamper": { "checks": trials_per_family, "detected": tamper_detected },
            "latency_ms": {
                "p50": percentile(&latencies_ms, 0.50),
                "p95": percentile(&latencies_ms, 0.95),
                "p99": percentile(&latencies_ms, 0.99)
            }
        }));
    }

    let elapsed = run_started.elapsed();
    let vertical_macro_rate = if family_false_admission_rates.is_empty() {
        Value::Null
    } else {
        json!(family_false_admission_rates.iter().sum::<f64>() / family_false_admission_rates.len() as f64)
    };
    let vertical_macro_upper = if family_simultaneous_upper_bounds.is_empty() {
        Value::Null
    } else {
        json!(family_simultaneous_upper_bounds.iter().sum::<f64>() / family_simultaneous_upper_bounds.len() as f64)
    };
    let corpus_sha256 = corpus_hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();
    let base_profile_sha256 = digest_value(&profile);
    let base_pin_sha256 = digest_value(&pin);
    let base_request_sha256 = digest_value(&request);
    let report = json!({
        "schema_version": "luminous.ieac-runtime-admission-benchmark-report.v0.1",
        "benchmark_id": manifest["benchmark"]["benchmark_id"],
        "manifest_id": manifest["manifest_id"],
        "manifest_exact_bytes_sha256": manifest_sha256,
        "manifest_preregistered": true,
        "vertical": vertical,
        "prng": "splitmix64-v1",
        "corpus_sha256": corpus_sha256,
        "input_fixture_digests": {
            "profile_sha256": base_profile_sha256,
            "profile_pin_sha256": base_pin_sha256,
            "request_sha256": base_request_sha256
        },
        "seed": seed,
        "trials_per_failure_family": trials_per_family,
        "candidate_method_id": candidate_method["method_id"],
        "static_baseline_method_id": baseline_method["method_id"],
        "evaluator_version": EVALUATOR_VERSION,
        "checker_version": CHECKER_VERSION,
        "claim_ceiling": "Synthetic software admission-control benchmark only. It does not validate physical models, field data, source truth, real-world risk, industrial outcomes, production readiness, certification or SOTA.",
        "summary": {
            "failure_family_count": FAMILIES.len(),
            "negative_trials": negative_trials_total,
            "false_admissions": false_admissions_total,
            "negative_rejects": negative_rejects_total,
            "negative_abstentions": negative_abstentions_total,
            "negative_non_admission_rate": if negative_trials_total > 0 { json!((negative_rejects_total + negative_abstentions_total) as f64 / negative_trials_total as f64) } else { Value::Null },
            "positive_trials": positive_trials_total,
            "positive_admissions": positive_admissions_total,
            "positive_admission_coverage": if positive_trials_total > 0 { json!(positive_admissions_total as f64 / positive_trials_total as f64) } else { Value::Null },
            "positive_abstentions": positive_abstentions_total,
            "positive_rejections": positive_rejections_total,
            "negative_abstention_share": if negative_trials_total > 0 { json!(negative_abstentions_total as f64 / negative_trials_total as f64) } else { Value::Null },
            "static_baseline_counterfactual_acceptances": static_baseline_counterfactual_acceptances_total,
            "equal_weight_failure_family_macro_false_admission_rate": vertical_macro_rate,
            "one_sided_95_percent_false_admission_upper_bound_per_family": "reported_in_each_stratum",
            "one_sided_95_percent_upper_bound_for_equal_weight_family_macro_average": vertical_macro_upper,
            "family_macro_upper_bound_method": "Bonferroni simultaneous one-sided exact binomial bounds across negative families, averaged with equal family weights",
            "candidate_checker_disagreements": checker_disagreements_total,
            "expected_disposition_mismatches": disposition_mismatches_total,
            "expected_reason_code_misses": reason_code_misses_total,
            "report_tamper_checks": tamper_checks_total,
            "report_tamper_detected": tamper_detected_total,
            "deterministic_replay_checks": replay_checks,
            "deterministic_replay_mismatches": replay_mismatches,
            "elapsed_seconds": elapsed.as_secs_f64(),
            "throughput_cases_per_second": (FAMILIES.len() * trials_per_family) as f64 / elapsed.as_secs_f64().max(1e-9),
            "end_to_end_latency_ms": {
                "p50": percentile(&all_latencies_ms, 0.50),
                "p95": percentile(&all_latencies_ms, 0.95),
                "p99": percentile(&all_latencies_ms, 0.99)
            },
            "peak_memory": "not_measured",
            "cpu_energy": "not_measured"
        },
        "strata": strata
    });

    if let Some(path) = output_path {
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|e| format!("cannot serialize report: {e}"))?;
        write_new(&path, &bytes)?;
    }

    let passed = false_admissions_total == 0
        && checker_disagreements_total == 0
        && disposition_mismatches_total == 0
        && reason_code_misses_total == 0
        && tamper_detected_total == tamper_checks_total
        && replay_mismatches == 0;
    Ok((report, passed))
}

fn main() {
    match run() {
        Ok((report, passed)) => {
            println!("{}", serde_json::to_string_pretty(&report).expect("benchmark report serializes"));
            if !passed {
                eprintln!("benchmark acceptance gate failed");
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!("usage: industry-admission-benchmark --manifest <frozen-ibap-manifest.json> [--trials-per-family N (must match manifest)] [--seed N|0xHEX (must match manifest)] [--output <new-report.json>]");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_event_exact_upper_bound_matches_rule_of_three_scale() {
        let bound = one_sided_exact_upper(0, 30_000, 0.05);
        assert!(bound > 0.000099 && bound < 0.000101, "{bound}");
    }

    #[test]
    fn exact_upper_bound_increases_with_observed_false_admissions() {
        let zero = one_sided_exact_upper(0, 1_000, 0.05);
        let one = one_sided_exact_upper(1, 1_000, 0.05);
        assert!(one > zero, "zero={zero}, one={one}");
    }

    fn minimal_frozen_manifest() -> Value {
        json!({
            "schema_version": "luminous.industrial-benchmark-run-manifest.v0.1",
            "benchmark": {
                "evaluation_mode": "synthetic_fixture",
                "vertical": "industrial_maintenance"
            },
            "freeze": { "pre_registered_before_execution": true },
            "methods": [
                { "method_id": "baseline", "role": "null_baseline" },
                { "method_id": "candidate", "role": "candidate" }
            ],
            "claim_ceiling": "synthetic_only",
            "comparability": { "state": "matched" },
            "replication": {
                "replicate_count": 1,
                "seeds": [20261031],
                "trials_per_stratum": 100
            }
        })
    }

    #[test]
    fn frozen_manifest_drives_vertical_trial_count_and_seed() {
        let manifest = minimal_frozen_manifest();
        let (vertical, trials, seed) = validate_manifest(&manifest).expect("valid frozen fixture");
        assert_eq!(vertical, "industrial_maintenance");
        assert_eq!(trials, 100);
        assert_eq!(seed, 20_261_031);
    }

    #[test]
    fn cli_cannot_change_preregistered_trial_count_or_seed() {
        assert!(validate_cli_overrides(Some(99), None, 100, 20_261_031).is_err());
        assert!(validate_cli_overrides(None, Some(17), 100, 20_261_031).is_err());
        assert!(validate_cli_overrides(Some(100), Some(20_261_031), 100, 20_261_031).is_ok());
        assert!(validate_cli_overrides(None, None, 100, 20_261_031).is_ok());
    }

    #[test]
    #[test]
    fn expected_reason_codes_require_exact_set_equality() {
        assert!(!reason_codes_match(&json!({}), &[]));
        assert!(!reason_codes_match(&json!({ "reason_codes": "not-an-array" }), &[]));
        assert!(reason_codes_match(
            &json!({ "reason_codes": ["ARTIFACT_REVOKED", "MEASUREMENT_STALE"] }),
            &["MEASUREMENT_STALE", "ARTIFACT_REVOKED"],
        ));
        assert!(!reason_codes_match(
            &json!({ "reason_codes": ["ARTIFACT_REVOKED", "MEASUREMENT_STALE", "UNEXPECTED"] }),
            &["MEASUREMENT_STALE", "ARTIFACT_REVOKED"],
        ));
    }

    #[test]
    fn splitmix_seed_replays_exactly() {
        let mut left = Rng::new(0x2026_1031);
        let mut right = Rng::new(0x2026_1031);
        for _ in 0..100 {
            assert_eq!(left.next(), right.next());
        }
    }
}

