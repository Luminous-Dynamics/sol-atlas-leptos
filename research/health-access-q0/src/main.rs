use health_access_q0::{audit_corpus, parse_and_verify_corpus, sha256};
use serde_json::json;
use std::{env, fs, process};

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/reference-cases.v0.json".to_string());
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("BLOCKED: cannot read fixture {path}: {error}");
        process::exit(2);
    });
    let fixture_sha256 = sha256(&bytes);
    let corpus = parse_and_verify_corpus(&bytes).unwrap_or_else(|error| {
        eprintln!("FAIL: {error}");
        process::exit(1);
    });
    let results = audit_corpus(&corpus).unwrap_or_else(|error| {
        eprintln!("FAIL: audit error: {error}");
        process::exit(1);
    });

    for (case, result) in corpus.cases.iter().zip(results.iter()) {
        if result.eligible_facility_ids != case.expected.eligible_facility_ids
            || result.reachable_capacity_units != case.expected.reachable_capacity_units
            || result.served_units != case.expected.served_units
            || result.unserved_units != case.expected.unserved_units
        {
            eprintln!("FAIL: frozen oracle mismatch for {}", case.id);
            process::exit(1);
        }
    }

    let revision = env::var("HEALTH_ACCESS_CODE_REVISION")
        .or_else(|_| env::var("GITHUB_SHA"))
        .unwrap_or_else(|_| "unknown".to_string());
    println!(
        "{}",
        json!({
            "schema_version": "health-access-q0-receipt-v0.1",
            "status": "PASS",
            "code_revision": revision,
            "fixture_path": path,
            "fixture_sha256": fixture_sha256,
            "case_count": results.len(),
            "claim_ceiling": "Synthetic contract correctness only; no real-world facility, clinical, or health-outcome claim."
        })
    );
}
