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

Both baseline query graphs begin with all task statuses pending. Each shape measures validation, topological ordering (which includes validation), baseline ready-task derivation, index compilation, and compiled ready-frontier lookup. Version 4 additionally constructs a separate valid impact-analysis snapshot where `task-00000` is `verification_recorded`, then measures impact-index compilation, the reference graph's impact planner, and the compiled graph's impact planner. It records the number of downstream descendants in each graph shape. Five warmup iterations are excluded from each recorded distribution. Reference and compiled readiness queries return the same task-reference collection shape. Impact planners return the same compact predecessor-witness collection; each impact records its predecessor task ID and hop depth rather than copying a full path per descendant. Query timing excludes one-time index compilation; use separate compile distributions when calculating break-even query counts. Output includes OS/architecture, crate version, workload dimensions, initial frontier width, impact closure size, percentile method, and raw nanosecond samples; capture the full JSON with exact source SHA and `rustc -Vv`. Use a release build on an otherwise idle, named runner, run at least 100 samples and report raw distributions plus p50/p95/max. Repeat cold-start and warm runs separately rather than mixing them.

This is only an algorithm microbenchmark. It does not measure IndexedDB/SQLite durability, serialization/payload cost on realistic field records, UI rendering, network reconnection, authorization, or a commercial competitor. The first comparative questions are whether index compilation pays for itself at observed query counts and whether results hold for both tiny and wide frontiers; a faster compiled query is not a win if jobs are so small or short-lived that compilation cost dominates, and a narrow-frontier-only speedup is not a general graph-performance win. The version 3 ready-frontier baseline and version 4 impact-planner measurements are recorded below. Artifacts remain tied to their exact source SHAs. A measured microbenchmark is evidence only for its workload and runner, not proof that a general performance target or product outcome has been achieved.

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

## Release evidence workflow

Pull requests touching the task-graph model, runner, fixtures or benchmark method trigger `.github/workflows/task-graph-benchmark.yml`. It pins the runner image to Ubuntu 24.04, runs the release profile for both graph shapes (100 samples × 10,000 tasks), validates sample counts/edge counts/profile/schema, and uploads:
- `task-graph-benchmark.json`: complete raw latency samples and summary percentiles
- `task-graph-benchmark-summary.json`: compact per-operation p50/p95/max for both graph shapes
- `benchmark-environment.txt`: exact source SHA, Rust/Cargo identity, kernel and CPU information

The version 4 gate also requires 100 raw samples each for `impact_compile_index`, `impact_plan_reference`, and `impact_plan_compiled`; the fan-in graph must expose 9,999 reachable descendants from its verified root and independent roots must expose zero. Those checks validate benchmark corpus shape and evidence completeness, not the speed target itself. Artifacts measure the GitHub-hosted runner for the run, not universal hardware. Compare only after accounting for runner/CPU, toolchain, source SHA, graph shape, impact count and sample method. The workflow records measurements rather than failing against an unmeasured target; add regression thresholds after a stable baseline.

## First measured release baseline (2026-10-20)

This is the version 3 artifact captured by the release benchmark workflow, at exact source commit `b8def761e1fde6bb6550b3e93911a29e7b6e0c38`. [Workflow run 39407001243](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39407001243) passed benchmark execution and schema/sample-count validation; the run uploaded raw and summarized JSON plus environment metadata.

**Environment:** GitHub-hosted Ubuntu 24.04 runner, 4 reported vCPUs (AMD EPYC 7763 virtual machine), x86_64 Linux, Rust/Cargo 1.99.1, release profile, 5 warmup runs and 100 measured samples per operation. The artifact retains all raw nanosecond samples. These are measurements of this runner and source revision, not a universal performance guarantee.

| Graph shape | Tasks / edges | Initial ready set | Reference `ready_tasks` p50 / p95 | Compiled `compiled_ready_tasks` p50 / p95 | Observed p50 / p95 query ratio | Index compile p50 / p95 |
|---|---:|---:|---:|---:|---:|---:|
| `fan_in_4` | 10,000 / 39,990 | 1 | 21.911 ms / 24.300 ms | 50 ns / 60 ns | 438,212× / 405,005× | 28.838 ms / 29.359 ms |
| `independent_roots` | 10,000 / 0 | 10,000 | 5.466 ms / 5.568 ms | 48.071 µs / 58.340 µs | 113.71× / 95.44× | 7.822 ms / 7.913 ms |

The very large ratio in the narrow-frontier case is shape-specific: the reference path revalidates and rescans all 10,000 tasks while the compiled query returns one ready task from an already-built index. The independent-root case exercises a wide ready set and still shows a repeated-query reduction, but materializing 10,000 task references takes measurable time. These ratios compare the same returned task-reference collection and release build, but only for this synthetic implementation.

Using the observed p50 timings and treating compile cost as a one-time cost, the compiled representation pays for itself at approximately the **second** ready-frontier query for each tested shape. This estimate excludes index rebuilds caused by graph revision changes, persistence, serialization, browser/UI work and network operations. We have not yet measured incremental `apply` latency, correction-impact planning latency, memory use, realistic task-status distributions, or end-to-end mobile use.

**Interpretation:** this supports retaining the compiled frontier for sessions that issue multiple queries per graph revision. It does not prove a general win over a commercial field-service product or justify unconditional compilation for one-shot or rapidly changing graphs. Next benchmark improvements should include transition-heavy workloads, graph revision rebuild rates, allocation/memory accounting, and impact-plan closure costs.

## How to compare with industry fairly

Enterprise field-service products already support complex work orders and offline work. This proposal does not claim that Microsoft Dynamics 365, Salesforce, or ServiceNow lack task sequencing or parallel workflow features. The comparison is narrower: how well does a tested dependency graph preserve the distinction between pending, active, reported completion and recorded verification; how deterministic is the ready-action set; and how well does it avoid avoidable serialization?

Do not compare against vendor marketing descriptions or infer superiority from a unit test. Build the same frozen scenarios in the baseline and candidate, disclose each product/version/configuration, measure user-level outcomes and all invalid transitions, and publish failures as well as averages. Claim a win only on the metrics where equivalent evidence exists.

## Next steps before production

1. Agree on who owns canonical work-order identity, verification semantics and task-graph revision.
2. Bind tasks to typed evidence IDs, procedure revisions and role/authorization references in the authoritative record system.
3. Add rework edges and revisioned graph migration only after defining how previously completed downstream tasks are invalidated when a prerequisite is corrected.
4. Add a persistent graph/event adapter and crash/reconnect fault injection.
5. Connect the graph to Leptos as the accessible primary work list; let Bevy show spatial relationships without becoming necessary for completing a task.

## Version 4 measured impact-planner baseline (2026-10-20)

Exact source commit: `169f026dfaf1bad79db883efb25f3ef281936699`. [Release benchmark run 39408860198](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39408860198) passed the raw-evidence/schema checks and uploaded all samples plus environment metadata. The benchmark uses the release profile, 5 warmups, and 100 measured samples per operation.

**Environment:** Ubuntu 24.04 GitHub-hosted runner, 4 reported vCPUs, AMD EPYC 7763 virtual CPU, x86_64, Rust/Cargo 1.99.1, `debug_assertions=false`.

| Graph shape | Impact descendants | Reference impact plan p50 / p95 | Compiled impact plan p50 / p95 | p95 ratio | Compiled impact index build p50 / p95 |
|---|---:|---:|---:|---:|---:|
| `fan_in_4` | 9,999 | 26.696 ms / 26.939 ms | 2.420 ms / 2.467 ms | 10.92× | 28.878 ms / 29.337 ms |
| `independent_roots` | 0 | 4.733 ms / 4.819 ms | 0.301 µs / 0.351 µs | Not a meaningful closure comparison | 7.776 ms / 7.922 ms |

For the non-empty closure, both planners meet the initial 100 ms p95 hypothesis on this runner, and the compiled planner is approximately **10.9× faster at p95**. The reference planner rebuilds and sorts adjacency as part of each plan; the compiled planner reuses an already-built dependent index. The independent-root case is an empty-impact control, so its tiny compiled-plan latency must not be represented as performance on a non-empty correction closure.

The one-time impact index build cost p50 (28.878 ms) is slightly greater than the p50 saving per query (26.696 − 2.420 = 24.276 ms); by this simplified calculation, the index pays for itself at roughly the second impact query for the same graph revision. Rebuild frequency matters: if a graph changes before a second query, compilation may not pay back its cost.

This only times local data structures and plan construction. It excludes persistence, JSON serialization, DOM/rendering, transport, authorization, corrective-evidence verification and any real field-service workflow. No claim of fewer errors, less rework, safer physical work, or superiority to ServiceNow/Microsoft/Salesforce follows from this microbenchmark. Those require equivalent end-to-end scenarios, realistic status distributions, mobile measurements and independent product configuration.

