// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible in-memory benchmark for task evidence packet evaluation.
//! Example: cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_evidence_bench -- 100 1000 10000
//! Args: samples, requirement count, evidence count. No storage, signature validation,
//! canonical-byte digest recomputation, authentication, or network I/O is measured.

use sol_atlas_core::task_evidence::{
    EvidenceArtifactV1, EvidenceDecision, EvidenceKind, EvidenceRequirementV1,
    TASK_EVIDENCE_SCHEMA_VERSION, TaskEvidencePacketV1, VerificationContext,
    evaluate_evidence_packet,
};
use std::{env, hint::black_box, time::Instant};

fn kind(index: usize) -> EvidenceKind {
    match index % 6 {
        0 => EvidenceKind::Observation,
        1 => EvidenceKind::Measurement,
        2 => EvidenceKind::Photograph,
        3 => EvidenceKind::TestResult,
        4 => EvidenceKind::ProcedureChecklist,
        _ => EvidenceKind::ImportedRecord,
    }
}

fn build(requirements_count: usize, evidence_count: usize) -> TaskEvidencePacketV1 {
    let requirements: Vec<_> = (0..requirements_count)
        .map(|i| EvidenceRequirementV1 {
            requirement_id: format!("requirement-{i:05}"),
            accepted_kind: kind(i),
            required_procedure_revision: Some(7),
            requires_independent_reviewer: i % 5 == 0,
        })
        .collect();
    let evidence = (0..evidence_count)
        .map(|i| {
            let req = i % requirements_count;
            let evidence_kind = requirements[req].accepted_kind;
            EvidenceArtifactV1 {
                evidence_id: format!("evidence-{i:06}"),
                requirement_id: requirements[req].requirement_id.clone(),
                kind: evidence_kind,
                source_ref: format!("urn:luminous:synthetic:evidence-{i:06}"),
                digest_sha256: format!("{:064x}", i as u128 + 1),
                recorded_by: "technician-01".to_owned(),
                captured_at_unix_ms: Some(1_797_000_000_000 + i as u64),
                unit: (evidence_kind == EvidenceKind::Measurement).then(|| "kPa".to_owned()),
                instrument_ref: (evidence_kind == EvidenceKind::Measurement)
                    .then(|| format!("instrument:ref-{req:05}")),
                procedure_revision: Some(7),
            }
        })
        .collect();
    TaskEvidencePacketV1 {
        schema_version: TASK_EVIDENCE_SCHEMA_VERSION,
        graph_id: "benchmark:evidence-v1".to_owned(),
        graph_revision: 4,
        task_id: "benchmark:functional-test".to_owned(),
        procedure_revision: 7,
        prepared_by: "technician-01".to_owned(),
        requirements,
        evidence,
    }
}

fn percentile(sorted: &[u128], numerator: usize) -> u128 {
    let index = (sorted.len() * numerator).div_ceil(100).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 3 {
        return Err("usage: task_evidence_bench [samples] [requirements] [evidence]".into());
    }
    let parse = |i: usize, name: &str, default: usize| -> Result<usize, String> {
        args.get(i).map_or(Ok(default), |v| {
            v.parse().map_err(|_| format!("invalid {name}: {v}"))
        })
    };
    let samples = parse(0, "sample count", 30)?;
    let req_count = parse(1, "requirement count", 1_000)?;
    let evidence_count = parse(2, "evidence count", 10_000)?;
    if !(1..=10_000).contains(&samples)
        || !(1..=100_000).contains(&req_count)
        || evidence_count < req_count
        || evidence_count > 1_000_000
    {
        return Err("samples must be 1..=10000, requirements 1..=100000, and evidence >= requirements and <= 1000000".into());
    }

    let packet = build(req_count, evidence_count);
    let context = VerificationContext {
        graph_id: "benchmark:evidence-v1",
        graph_revision: 4,
        task_id: "benchmark:functional-test",
        expected_procedure_revision: 7,
        expected_requirements: packet.requirements.clone(),
        reviewer_id: Some("reviewer-02"),
    };
    for _ in 0..5 {
        let result = evaluate_evidence_packet(black_box(&packet), &context);
        if result.decision != EvidenceDecision::ReadyForQualifiedReview {
            return Err(format!(
                "generated packet failed validation: {:?}",
                result.issues.first()
            ));
        }
        black_box(result);
    }

    let mut raw_ns = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        let result = evaluate_evidence_packet(black_box(&packet), &context);
        raw_ns.push(start.elapsed().as_nanos());
        if result.decision != EvidenceDecision::ReadyForQualifiedReview {
            return Err(format!(
                "generated packet failed validation: {:?}",
                result.issues.first()
            ));
        }
        black_box(result);
    }
    let mut sorted = raw_ns.clone();
    sorted.sort_unstable();
    let output = serde_json::json!({
        "benchmark": "task_evidence_packet_evaluation_v1",
        "schema_version": TASK_EVIDENCE_SCHEMA_VERSION,
        "requirements": req_count,
        "evidence_records": evidence_count,
        "warmup_samples": 5,
        "measured_samples": samples,
        "host": {"os": env::consts::OS, "architecture": env::consts::ARCH},
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "decision": "ready_for_qualified_review",
        "completeness_percent": 100,
        "latency_ns": {
            "p50": percentile(&sorted, 50),
            "p95": percentile(&sorted, 95),
            "max": sorted.last().copied().unwrap_or_default(),
            "raw": raw_ns
        }
    });
    println!(
        "{}",
        serde_json::to_string(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() -> Result<(), String> {
    run()
}
