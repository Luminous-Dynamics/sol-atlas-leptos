//! Deterministic synthetic CTER relay scale profile.
//!
//! This binary reports host measurements for synthetic Mycelix-shaped event
//! envelopes only. Event hashes and payload digests are deterministic fixture
//! identifiers, not computed BLAKE3/SHA-256 values. It does not measure radio,
//! Android, battery, cryptographic verification or legal/consent operations.
//!
//! Usage:
//!   cargo run --release --bin scale_profile -- <unique_events> <streams> <duplicate_percent> <forward|reverse|shuffle>
//!
//! Example:
//!   cargo run --release --bin scale_profile -- 10000 1000 25 reverse

use cter_relay_reference::evaluate;
use serde_json::json;
use std::env;
use std::time::Instant;

fn parse_arg<T: std::str::FromStr>(args: &[String], index: usize, default: T) -> T {
    args.get(index).and_then(|value| value.parse::<T>().ok()).unwrap_or(default)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let unique_events: usize = parse_arg(&args, 1, 10_000);
    let stream_count: usize = parse_arg(&args, 2, 1);
    let duplicate_percent: usize = parse_arg(&args, 3, 0);
    let arrival_order = args.get(4).cloned().unwrap_or_else(|| "forward".to_owned());
    if !matches!(arrival_order.as_str(), "forward" | "reverse" | "shuffle") {
        eprintln!("arrival order must be forward, reverse, or shuffle");
        std::process::exit(2);
    }
    let stream_count = stream_count.max(1);
    let duplicate_percent = duplicate_percent.min(100);

    let mut events = Vec::with_capacity(unique_events + unique_events * duplicate_percent / 100);
    let mut previous_hashes = vec![None::<String>; stream_count];
    let mut sequences = vec![0_u64; stream_count];

    for id in 0..unique_events {
        let stream = id % stream_count;
        let sequence = sequences[stream];
        sequences[stream] += 1;
        let event_hash = format!("{:064x}", id + 1);
        let payload_hash = format!("{:064x}", id + unique_events + 1);
        let stream_id = format!("stream-{stream}");
        let delivery = json!({
            "event": {
                "event_id": format!("event-{id}"),
                "stream_id": stream_id,
                "sequence": sequence,
                "previous_hash": previous_hashes[stream],
                "event_hash": event_hash,
                "actor": format!("actor-{stream}")
            },
            "assessment": {
                "assessment_id": format!("assessment-{id}"),
                "verifier_id": "synthetic-verifier",
                "policy_id": "synthetic-key-policy-v0",
                "assessed_at": "2026-11-19T00:00:00Z",
                "key_status_snapshot_ref": "urn:synthetic:key-status:snapshot:1",
                "event_time_basis": "trusted_append_receipt",
                "time_evidence_ref": format!("urn:synthetic:time-evidence:{id}"),
                "status": "verified"
            },
            "context": {
                "projection_key": "synthetic:shared-projection",
                "access_class": "restricted"
            },
            "payload": {
                "blob_ref": format!("cas://synthetic/{id}"),
                "media_type": "application/vnd.mycelix.signed-scientific-event+json",
                "blob_sha256": payload_hash
            }
        });
        previous_hashes[stream] = Some(event_hash);
        events.push(delivery);
    }

    let duplicates = unique_events * duplicate_percent / 100;
    for index in 0..duplicates {
        events.push(events[index].clone());
    }
    match arrival_order.as_str() {
        "reverse" => events.reverse(),
        "shuffle" => {
            // Fixed-seed Fisher-Yates shuffle; no external RNG or nondeterminism.
            let mut state = 0x4354_4552_u64;
            for index in (1..events.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let other = (state as usize) % (index + 1);
                events.swap(index, other);
            }
        }
        _ => {}
    }

    let started = Instant::now();
    let result = evaluate(&events);
    let elapsed = started.elapsed();
    let seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    let report = json!({
        "schema": "luminous.cter.scale-result.v0",
        "synthetic_only": true,
        "unique_events": unique_events,
        "deliveries": events.len(),
        "stream_count": stream_count,
        "duplicate_percent": duplicate_percent,
        "arrival_order": arrival_order,
        "accepted_events": result.accepted_events.len(),
        "conflict_groups": result.conflict_groups.len(),
        "target_os": std::env::consts::OS,
        "target_arch": std::env::consts::ARCH,
        "elapsed_micros": elapsed.as_micros() as u64,
        "deliveries_per_second": events.len() as f64 / seconds,
        "claim_ceiling": "host synthetic evaluator timing only; no radio, energy, cryptography, legal or field claim"
    });
    println!("{report}");
}
