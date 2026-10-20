// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Reproducible microbenchmark for the field task graph.
//!
//! Run with:
//! cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_graph_bench
//! cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_graph_bench -- 100 10000
//!
//! The first argument is measured iterations per operation; the second is task
//! count. This reports raw nanosecond samples and percentile summaries as JSON.
//! It does not benchmark durable storage, UI rendering, server authorization,
//! or competing vendor products.

use sol_atlas_core::task_graph::{TASK_GRAPH_SCHEMA_VERSION, TaskGraphV1, TaskNodeV1, TaskStatus};
use std::env;
use std::hint::black_box;
use std::time::Instant;

const DEFAULT_SAMPLES: usize = 30;
const DEFAULT_TASKS: usize = 10_000;
const MAX_SAMPLES: usize = 10_000;
const MAX_TASKS: usize = 100_000;
const DEPENDENCY_FAN_IN: usize = 4;
const WARMUP_RUNS: usize = 5;

fn parse_arg<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    name: &str,
    default: T,
) -> Result<T, String> {
    match args.next() {
        Some(value) => value
            .parse::<T>()
            .map_err(|_| format!("invalid {name}: {value}")),
        None => Ok(default),
    }
}

fn build_graph(task_count: usize) -> TaskGraphV1 {
    let tasks = (0..task_count)
        .map(|index| {
            let lower = index.saturating_sub(DEPENDENCY_FAN_IN);
            let depends_on = (lower..index)
                .map(|dependency| format!("task-{dependency:05}"))
                .collect();

            TaskNodeV1 {
                task_id: format!("task-{index:05}"),
                title: format!("Synthetic task {index}"),
                depends_on,
                priority: ((index * 37) % 101) as u8,
                status: TaskStatus::Pending,
            }
        })
        .collect();

    TaskGraphV1 {
        schema_version: TASK_GRAPH_SCHEMA_VERSION,
        graph_id: "benchmark:task-graph-v1:fan-in-4".to_owned(),
        revision: 1,
        tasks,
    }
}

fn measure(graph: &TaskGraphV1, samples: usize, operation: &str) -> Result<Vec<u128>, String> {
    let mut raw_ns = Vec::with_capacity(samples);

    for iteration in 0..(samples + WARMUP_RUNS) {
        let start = Instant::now();
        match operation {
            "validate" => graph.validate().map_err(|error| error.to_string())?,
            "topological_order" => {
                let result = graph
                    .topological_order()
                    .map_err(|error| error.to_string())?;
                black_box(result);
            }
            "ready_tasks" => {
                let result = graph.ready_tasks().map_err(|error| error.to_string())?;
                black_box(result.len());
            }
            _ => return Err(format!("unknown benchmark operation: {operation}")),
        }
        let elapsed_ns = start.elapsed().as_nanos();
        if iteration >= WARMUP_RUNS {
            raw_ns.push(elapsed_ns);
        }
    }

    Ok(raw_ns)
}

fn percentile(sorted: &[u128], numerator: usize, denominator: usize) -> u128 {
    debug_assert!(!sorted.is_empty());
    debug_assert!(denominator > 0);
    let rank = sorted.len() * numerator;
    let index = rank.div_ceil(denominator).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn summary(raw_ns: &[u128]) -> serde_json::Value {
    let mut sorted = raw_ns.to_vec();
    sorted.sort_unstable();
    serde_json::json!({
        "samples": raw_ns.len(),
        "p50_ns": percentile(&sorted, 50, 100),
        "p95_ns": percentile(&sorted, 95, 100),
        "max_ns": sorted.last().copied().unwrap_or_default(),
        "raw_ns": raw_ns,
    })
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let samples: usize = parse_arg(&mut args, "sample count", DEFAULT_SAMPLES)?;
    let task_count: usize = parse_arg(&mut args, "task count", DEFAULT_TASKS)?;
    if args.next().is_some() {
        return Err("usage: task_graph_bench [sample-count] [task-count]".to_owned());
    }
    if !(1..=MAX_SAMPLES).contains(&samples) {
        return Err(format!("sample count must be between 1 and {MAX_SAMPLES}"));
    }
    if !(1..=MAX_TASKS).contains(&task_count) {
        return Err(format!("task count must be between 1 and {MAX_TASKS}"));
    }

    let graph = build_graph(task_count);
    graph.validate().map_err(|error| error.to_string())?;
    let edge_count: usize = graph.tasks.iter().map(|task| task.depends_on.len()).sum();
    let topological = graph
        .topological_order()
        .map_err(|error| error.to_string())?;
    if topological.len() != task_count {
        return Err("topological output length did not match task count".to_owned());
    }

    let operations = ["validate", "topological_order", "ready_tasks"];
    let mut results = serde_json::Map::new();
    for operation in operations {
        let raw = measure(&graph, samples, operation)?;
        results.insert(operation.to_owned(), summary(&raw));
    }

    let output = serde_json::json!({
        "benchmark": "field-task-graph-v1",
        "benchmark_version": 1,
        "claim_ceiling": "local algorithm microbenchmark only; not end-to-end field-service or vendor comparison",
        "environment": {
            "os": env::consts::OS,
            "arch": env::consts::ARCH,
            "rust_package_version": env!("CARGO_PKG_VERSION"),
            "debug_assertions": cfg!(debug_assertions),
            "release_profile_recommended": true
        },
        "dataset": {
            "graph_id": graph.graph_id,
            "schema_version": graph.schema_version,
            "tasks": task_count,
            "edges": edge_count,
            "dependency_fan_in_max": DEPENDENCY_FAN_IN,
            "seed": "deterministic-index-derived-no-rng",
            "all_task_statuses": "pending"
        },
        "method": {
            "samples_per_operation": samples,
            "clock": "std::time::Instant",
            "unit": "nanoseconds",
            "percentiles": "nearest-rank",
            "warmup_runs": WARMUP_RUNS,
            "raw_samples_retained": true
        },
        "operations": results
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("task_graph_bench: {error}");
        std::process::exit(2);
    }
}
