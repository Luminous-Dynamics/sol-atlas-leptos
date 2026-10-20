# Field Task Graph v1: from universal checklist to dependency-aware work

**Status:** pure Rust graph/reducer prototype and synthetic fixture; not a production workflow or safety authority  
**Module:** `sol-atlas-core/src/task_graph.rs`  
**Fixture:** `sol-atlas-core/fixtures/task-graph-v1.json`

## Why change the model?

A linear workflow assumes every trade follows the same sequence. Real jobs may have parallel prerequisites: site access, inspection, procurement, specialist review, customer decisions, rework, and final verification. A static checklist either serializes independent work unnecessarily or tempts the UI to skip prerequisites.

Task Graph v1 represents prerequisites explicitly and derives readiness. Two tasks can be ready at the same time; a downstream task remains blocked until every prerequisite has `verification_recorded`. A `completion_reported` state never unlocks its dependents.

## Contract

- Schema version, graph identity and revision are explicit.
- Each task has a stable ID, human-readable title, prerequisite IDs, priority, and distinct status.
- IDs and dependency edges are validated; missing IDs, duplicate edges, self-dependencies and cycles are rejected.
- Topological ordering is deterministic, with task IDs breaking independent-node ties.
- Ready tasks are derived from the graph and sorted by descending priority then task ID.
- Every accepted local transition increments the projection revision exactly once; invalid transitions leave the graph unchanged.
- Cancellation never satisfies another task's prerequisite.

This API is a local planning reducer only. It does not establish that site-access permission is valid, that a worker is qualified, that physical work is safe, or that a server accepted a transition. Canonical service authorization, revision checks and evidence validation must independently gate real work.

## Frozen acceptance corpus

The JSON fixture expresses a synthetic access-approved job with parallel inspection and parts branches, followed by a functional test and handover. Unit tests cover fixture deserialization, deterministic ordering, parallel readiness, completion-versus-verification gating, invalid transition non-mutation, dependency integrity, cancellation, and revision increments.

## Reproducible algorithm runner

A Rust example generates a deterministic graph and emits machine-readable latency samples. It now compares repeated queries over the plain reference graph with a compiled runtime frontier, and measures index compilation separately:

```sh
cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_graph_bench
# optional: 100 samples per operation, 10,000 tasks
cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_graph_bench -- 100 10000
```

The runner measures two graph shapes at the requested task count (default 10,000), with 100 samples per operation by default:

- `fan_in_4`: a narrow, deterministic dependency frontier with maximum fan-in of four and 39,990 edges at 10,000 tasks.
- `independent_roots`: a wide frontier of 10,000 independent ready tasks and zero edges.

Both begin with all task statuses pending. Each shape measures validation, topological ordering (which includes validation), baseline ready-task derivation, index compilation, and compiled ready-frontier lookup. Five warmup iterations are excluded from each recorded distribution. Reference and compiled queries now return the same task-reference collection shape. Compiled query timing excludes the one-time index build; use the separate `compile_index` distribution when calculating break-even query count. Output includes OS/architecture, crate version, workload dimensions, initial frontier width, percentile method, and raw nanosecond samples; capture the full JSON together with the exact source SHA and `rustc -Vv`. Use a release build on an otherwise idle, named runner, run at least 100 samples and report raw distributions plus p50/p95/max. Repeat cold-start and warm runs separately rather than mixing them.

This is only an algorithm microbenchmark. It does not measure IndexedDB/SQLite durability, serialization/payload cost on realistic field records, UI rendering, network reconnection, authorization, or a commercial competitor. The first comparative questions are whether index compilation pays for itself at observed query counts and whether results hold for both tiny and wide frontiers; a faster compiled query is not a win if jobs are so small or short-lived that compilation cost dominates, and a narrow-frontier-only speedup is not a general graph-performance win. There is no result recorded in this document: the runner is a measurement mechanism, not evidence that the numerical target has been met.

## Benchmark plan

Run candidate and linear-baseline scenarios over the same frozen graph set, device, Rust build, and task scripts. The numerical thresholds below are initial hypotheses until measured.

| ID | Measure | Initial target / gate | Evidence required |
|---|---|---|---|
| DAG-01 | Invalid dependency graph accepted | 0 | Missing/duplicate/self/cyclic edges across the frozen negative corpus |
| DAG-02 | Downstream task unlocked by reported completion only | 0 | State/action matrix; only verification-recorded prerequisites release a task |
| DAG-03 | Deterministic ready set | 100% identical for identical graph revision | Golden vectors and repeated execution |
| DAG-04 | Illegal transition changes state/revision | 0 | Exhaustive task-state/action matrix; pre/post digest equality for rejected transitions |
| DAG-05 | Independent branch utilization | At least 2 concurrently ready tasks in the parallel reference scenario | Same task graph executed in linear and DAG prototypes; count avoidable serialization |
| DAG-06 | Task graph verification time | p95 under 50 ms for 10,000 tasks / 40,000 dependency edges (hypothesis) | Release build, pinned runner, fixed seeded graph generator, 1,000 samples, warm/cold runs, raw percentile method |
| DAG-07 | Incremental ready-frontier correctness | 100% same ordered ready IDs as reference reducer after every accepted action | Deterministic multi-branch transition traces and state-by-state parity tests |
| PERF-02 | Repeated ready queries | ≥3× p95 query-speed improvement after index compilation, with separately reported compile cost (hypothesis) | Same graph/revision, same runner, release build, 1,000 samples; calculate observed break-even query count |
| PERF-03 | Incremental transition cost | p95 under 1 ms for a verification transition with ≤100 direct dependents (hypothesis) | Generated fan-out cases, pinned release runner, raw samples, no storage/network in timed section |
| UX-01 | Time to resume after interruption | ≥30% lower median than linear baseline (hypothesis) | Paired simulated field tasks, with correctness/evidence checks unchanged |
| UX-02 | Readiness comprehension | ≥95% correct interpretation of why a task is blocked and what evidence can unlock it | Moderated task study by trade and experience |

## Industry comparison and the actual gap

ServiceNow Field Service already supports explicit task dependencies, including finish-to-start, start-after-start, and start-together relationships, plus minimum/maximum lag. It also supports multiple task agents and offline work. Microsoft Dynamics 365 Field Service uses incident types to standardize groups of service tasks, products, and services. Therefore, the existence of a DAG or multiple ready tasks is not itself an industry breakthrough. See [ServiceNow: Create dependencies between work order tasks](https://www.servicenow.com/docs/r/field-service-management/work-order-management/t_SetAnUpstreamTask.html), [ServiceNow: Manage work order tasks](https://www.servicenow.com/docs/r/field-service-management/work-order-management/c_WorkOrderTasks.html), and [Microsoft: Work order incident type overview](https://learn.microsoft.com/en-us/dynamics365/field-service/incident-type-overview).

Our narrow candidate is a deterministic, verifiable client-side ready frontier whose runtime index can be tested for parity and incremental transition cost, paired with evidence/verification semantics and a separate offline journal. That is an implementation hypothesis, not a claim of superiority over these products; v1 still lacks lag constraints, resource/skill scheduling, rework invalidation, canonical record integration and durable storage.

## How to compare with industry fairly

Enterprise field-service products already support complex work orders and offline work. This proposal does not claim that Microsoft Dynamics 365, Salesforce, or ServiceNow lack task sequencing or parallel workflow features. The comparison is narrower: how well does a tested dependency graph preserve the distinction between pending, active, reported completion and recorded verification; how deterministic is the ready-action set; and how well does it avoid avoidable serialization?

Do not compare against vendor marketing descriptions or infer superiority from a unit test. Build the same frozen scenarios in the baseline and candidate, disclose each product/version/configuration, measure user-level outcomes and all invalid transitions, and publish failures as well as averages. Claim a win only on the metrics where equivalent evidence exists.

## Next steps before production

1. Agree on who owns canonical work-order identity, verification semantics and task-graph revision.
2. Bind tasks to typed evidence IDs, procedure revisions and role/authorization references in the authoritative record system.
3. Add rework edges and revisioned graph migration only after defining how previously completed downstream tasks are invalidated when a prerequisite is corrected.
4. Add a persistent graph/event adapter and crash/reconnect fault injection.
5. Connect the graph to Leptos as the accessible primary work list; let Bevy show spatial relationships without becoming necessary for completing a task.
