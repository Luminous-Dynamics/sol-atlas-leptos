use energy_dispatch_schedule_auditor::{audit_schedule_bytes, parse_unique_json, AuditDisposition};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::hint::black_box;
use std::time::Instant;

const H2_REQUEST: &[u8] = include_bytes!("../fixtures/h2-request.json");
const WARMUPS: usize = 50;
const SAMPLES: usize = 1_000;

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn step(
    id: &str,
    charge: u16,
    withdrawal: u16,
    soc: u16,
    essential_served: u16,
    pv_used: u16,
    pv_curtailed: u16,
) -> Value {
    json!({
        "interval_id": id,
        "grid_import_quanta": 0,
        "charge_input_quanta": charge,
        "discharge_withdrawal_quanta": withdrawal,
        "soc_after_quanta": soc,
        "essential_served_quanta": essential_served,
        "essential_unserved_quanta": 0,
        "flexible_served_quanta": 0,
        "flexible_unserved_quanta": 0,
        "pv_used_quanta": pv_used,
        "pv_curtailed_quanta": pv_curtailed
    })
}

fn make_case(horizon: usize) -> (Vec<u8>, Vec<u8>) {
    assert!((2..=4).contains(&horizon));
    let mut request: Value = serde_json::from_slice(H2_REQUEST).expect("fixture request parses");
    let intervals = request["intervals"].as_array_mut().expect("interval array");
    if horizon >= 3 {
        intervals.push(json!({
            "interval_id": "post-outage-idle",
            "start_unix_seconds": 1_800_007_200_i64,
            "duration_seconds": 3600,
            "grid_outage": false,
            "pv_available_quanta": 0,
            "grid_import_limit_quanta": 0,
            "essential_demand_quanta": 0,
            "flexible_demand_quanta": 0,
            "grid_cost_units_per_quantum": 3,
            "grid_carbon_grams_per_quantum": 7
        }));
    }
    if horizon >= 4 {
        intervals.push(json!({
            "interval_id": "post-outage-recovery",
            "start_unix_seconds": 1_800_010_800_i64,
            "duration_seconds": 3600,
            "grid_outage": false,
            "pv_available_quanta": 1,
            "grid_import_limit_quanta": 1,
            "essential_demand_quanta": 0,
            "flexible_demand_quanta": 0,
            "grid_cost_units_per_quantum": 3,
            "grid_carbon_grams_per_quantum": 7
        }));
    }
    let request_bytes = serde_json::to_vec(&request).expect("serialize request");

    let mut dispatches = vec![
        step("charge-from-pv", 1, 0, 1, 0, 1, 0),
        step("serve-during-outage", 0, 1, 0, 1, 0, 0),
    ];
    if horizon >= 3 {
        dispatches.push(step("post-outage-idle", 0, 0, 0, 0, 0, 0));
    }
    if horizon >= 4 {
        dispatches.push(step("post-outage-recovery", 0, 0, 0, 0, 0, 1));
    }
    let schedule = json!({
        "schema_version": "energy-dispatch-schedule-v0.1",
        "case_id": "schedule-audit-fixture",
        "request_sha256": sha256(&request_bytes),
        "interval_dispatches": dispatches,
        "terminal_soc_quanta": 0
    });
    let schedule_bytes = serde_json::to_vec(&schedule).expect("serialize schedule");
    (request_bytes, schedule_bytes)
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    let index = ((sorted.len() - 1) * percentile).div_ceil(100);
    sorted[index]
}

fn measure<F>(mut operation: F) -> Vec<u128>
where
    F: FnMut(),
{
    for _ in 0..WARMUPS {
        operation();
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        operation();
        samples.push(started.elapsed().as_nanos());
    }
    samples.sort_unstable();
    samples
}

fn main() {
    println!("metric,horizon,samples,warmups,p50_ns,p95_ns,p99_ns,min_ns,max_ns,request_bytes,schedule_bytes");
    for horizon in 2..=4 {
        let (request, schedule) = make_case(horizon);
        let parse_samples = measure(|| {
            let request_value = parse_unique_json(black_box(&request)).expect("strict request parse");
            let schedule_value = parse_unique_json(black_box(&schedule)).expect("strict schedule parse");
            black_box((request_value, schedule_value));
        });
        let receipt_samples = measure(|| {
            let receipt = audit_schedule_bytes(black_box(&request), black_box(&schedule))
                .expect("schedule audit completes");
            assert_eq!(receipt.disposition, AuditDisposition::Accepted);
            black_box(receipt);
        });
        for (name, samples) in [
            ("strict_parse_only", parse_samples),
            ("end_to_end_audit_and_receipt", receipt_samples),
        ] {
            println!(
                "{name},{horizon},{SAMPLES},{WARMUPS},{},{},{},{},{},{},{}",
                percentile(&samples, 50),
                percentile(&samples, 95),
                percentile(&samples, 99),
                samples[0],
                samples[samples.len() - 1],
                request.len(),
                schedule.len()
            );
        }
    }
}
